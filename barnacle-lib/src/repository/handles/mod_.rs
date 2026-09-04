use std::{
    fmt::Debug,
    fs::{self, File},
    path::{Path, PathBuf},
};

use compress_tools::{Ownership, uncompress_archive};
use heck::ToSnakeCase;
use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};
use tracing::info;

use crate::{
    fs::{Permissions, change_dir_permissions},
    repository::{
        Cfg,
        db::{
            Db,
            models::mods::{ActiveModel, COLUMN, Entity, Model},
        },
        handles::{Error, Result, game::Game, map_insert_error},
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
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
    }

    async fn model(&self) -> Result<Model> {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    async fn active_model(&self) -> Result<ActiveModel> {
        Ok(self.model().await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .parent()
            .await?
            .dir()
            .await?
            .join("mods")
            .join(self.name().await?.to_snake_case()))
    }

    /// Returns the parent [`Game`] of this [`Mod`]
    pub async fn parent(&self) -> Result<Game> {
        let parent_game_id = self.model().await?.game_id;
        Ok(Game::from_id(
            parent_game_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    pub(crate) async fn add(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
        input_path: Option<&Path>,
    ) -> Result<Self> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|e| map_insert_error(e, Error::DuplicateModName(name.into())))?
            .last_insert_id;
        let mod_ = Mod::from_id(id, db.clone(), cfg.clone());

        // TODO: Only attempt to open the archive if the input_path is an archive
        if let Some(path) = input_path {
            let archive = File::open(path).unwrap();
            uncompress_archive(archive, &mod_.dir().await?, Ownership::Preserve).unwrap();
            change_dir_permissions(&mod_.dir().await?, Permissions::ReadOnly);
        } else {
            let path = mod_.dir().await?;
            fs::create_dir_all(path).unwrap();
        };

        info!("Added profile: {name}");

        Ok(mod_)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id))
            .order_by_id_desc()
            .all(db.conn())
            .await?
            .iter()
            .map(|model| Mod::from_id(model.id, db.clone(), cfg.clone()))
            .collect())
    }

    pub async fn remove(self) -> Result<()> {
        // We have to store these so we can still access them once the mod is deleted
        let name = self.name().await?;
        let dir = self.dir().await?;

        Entity::delete_by_id(self.id).exec(self.db.conn()).await?;

        fs::remove_dir_all(dir).unwrap();

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
    use crate::{
        Repository,
        repository::{DeployKind, handles::Error},
    };

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let mod_ = game.add_mod("Test", None).await.unwrap();

        assert!(mod_.dir().await.unwrap().exists());
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        game.add_mod("Test", None).await.unwrap();

        assert!(matches!(
            game.add_mod("Test", None).await,
            Err(Error::DuplicateModName(_))
        ))
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::mock().await;

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
        let repo = Repository::mock().await;
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
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let mod_ = game.add_mod("Test", None).await.unwrap();

        assert_eq!(mod_.parent().await.unwrap(), game);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::mock().await;

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
}
