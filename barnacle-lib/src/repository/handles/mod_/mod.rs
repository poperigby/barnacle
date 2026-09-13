use std::{
    fmt::Debug,
    fs::{self, File},
    path::{Path, PathBuf},
};

mod error;

use compress_tools::{Ownership, uncompress_archive};
use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::info;

pub use error::*;

use crate::{
    fs::{Permissions, change_dir_permissions},
    repository::{
        Cfg,
        db::{
            Db,
            models::mods::{ActiveModel, COLUMN, Entity, Model},
        },
        handles::{
            error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
            game::Game,
        },
    },
};

/// Represents a mod entity in the Barnacle system.
///
/// Provides methods to inspect and modify this mod's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Mod {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Mod {
    pub(crate) fn from_id(id: i32, db: &Db, cfg: &Cfg) -> Self {
        Self {
            id,
            db: db.clone(),
            cfg: cfg.clone(),
        }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Mod, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Mod, self.id))
    }

    // Fields

    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        Ok(self
            .parent()
            .await
            .map_err(DirError::Parent)?
            .dir()
            .await
            .map_err(DirError::ParentDir)?
            .join("mods")
            .join(self.id.to_string()))
    }

    /// Returns the parent [`Game`] of this [`Mod`]
    pub async fn parent(&self) -> Result<Game, ParentError> {
        let parent_game_id = self
            .model(self.db.conn())
            .await
            .map_err(ParentError::Load)?
            .game_id;

        Ok(Game::from_id(parent_game_id, &self.db, &self.cfg))
    }

    pub(crate) async fn add(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
        input_path: Option<&Path>,
    ) -> Result<Self, AddError> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id()),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    AddError::DuplicateName {
                        name: name.to_string(),
                    }
                } else {
                    AddError::Insert(source)
                }
            })?
            .last_insert_id;
        let mod_ = Mod::from_id(id, &db, &cfg);

        // TODO: Only attempt to open the archive if the input_path is an archive
        if let Some(path) = input_path {
            let archive = File::open(path).map_err(|source| AddError::OpenArchive {
                path: path.to_path_buf(),
                source,
            })?;
            let dir = mod_.dir().await.map_err(AddError::Dir)?;
            uncompress_archive(archive, &dir, Ownership::Preserve)
                .map_err(AddError::ExtractArchive)?;
            change_dir_permissions(&dir, Permissions::ReadOnly);
        } else {
            let dir = mod_.dir().await.map_err(AddError::Dir)?;
            fs::create_dir_all(&dir).map_err(|source| AddError::CreateDir { path: dir, source })?;
        };

        info!("Added mod: {name}");

        Ok(mod_)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Mod::from_id(model.id, db, cfg))
            .collect())
    }

    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the mod is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        fs::remove_dir_all(&dir).map_err(|source| RemoveError::RemoveDir { path: dir, source })?;

        info!("Removed mod: {name}");

        Ok(())
    }
}

impl PartialEq for Mod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use crate::{Repository, mod_, repository::DeployKind};

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let mod_ = game.add_mod("Test", None).await.unwrap();

        assert!(mod_.dir().await.unwrap().exists());
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        game.add_mod("Test", None).await.unwrap();

        assert!(matches!(
            game.add_mod("Test", None).await,
            Err(mod_::AddError::DuplicateName { .. })
        ))
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let mod_ = game.add_mod("Test", None).await.unwrap();

        assert_eq!(game.mods().await.unwrap().len(), 1);

        let dir = mod_.dir().await.unwrap();

        mod_.remove().await.unwrap();

        assert_eq!(game.mods().await.unwrap().len(), 0);
        assert!(!dir.exists())
    }

    #[tokio::test]
    async fn test_list() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();

        assert_eq!(game.mods().await.unwrap().len(), 0);

        game.add_mod("Better Spoon Textures 8K", None)
            .await
            .unwrap();

        assert_eq!(game.mods().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_parent() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let mod_ = game.add_mod("Test", None).await.unwrap();

        assert_eq!(mod_.parent().await.unwrap(), game);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::in_memory().await;

        repo.add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap()
            .add_mod("Test", None)
            .await
            .unwrap()
            .name()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_dir() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        let mod_ = game.add_mod("Test", None).await.unwrap();

        let expected_dir = repo
            .cfg
            .read()
            .library_dir()
            .join(game.id().to_string())
            .join("mods")
            .join(mod_.id.to_string());

        assert_eq!(mod_.dir().await.unwrap(), expected_dir);
    }
}
