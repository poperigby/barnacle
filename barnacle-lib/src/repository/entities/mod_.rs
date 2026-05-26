use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use compress_tools::{Ownership, uncompress_archive};
use heck::ToSnakeCase;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, ModelTrait};
use tracing::info;

use crate::{
    fs::{Permissions, change_dir_permissions},
    repository::{
        Cfg,
        db::{Db, models::mods},
        entities::{Error, Game, Result},
    },
};

#[derive(Debug, Clone)]
pub struct Mod {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Mod {
    pub(crate) async fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = mods::Entity::find_by_id(row_id).one(db.conn()).await?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    async fn model(&self) -> Result<mods::Model> {
        let model = mods::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?;
        model.ok_or(Error::RemovedEntity)
    }

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

    pub async fn parent(&self) -> Result<Game> {
        let game_id = self.model().await?.game_id;
        Game::load(game_id, self.db.clone(), self.cfg.clone()).await
    }

    pub(crate) async fn add(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
        path: Option<&Path>,
    ) -> Result<Self> {
        let model = mods::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            game_id: Set(game.id),
            name: Set(name.to_string()),
        };
        let inserted = model
            .insert(db.conn())
            .await
            .map_err(Error::from)?;

        let mod_ = Mod::load(inserted.id, db.clone(), cfg.clone()).await?;

        if let Some(path) = path {
            let archive = File::open(path).unwrap();
            uncompress_archive(archive, &mod_.dir().await?, Ownership::Preserve).unwrap();
            change_dir_permissions(&mod_.dir().await?, Permissions::ReadOnly);
        } else {
            fs::create_dir_all(mod_.dir().await?).unwrap();
        }

        Ok(mod_)
    }

    pub async fn remove(self) -> Result<()> {
        let name = self.name().await?;
        let dir = self.dir().await?;
        let Some(model) = mods::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
        else {
            return Err(Error::Internal(sea_orm::DbErr::Custom(
                "missing mod during delete".into(),
            )));
        };
        model.delete(self.db.conn()).await?;

        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }

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
        repository::{DeployKind, entities::Error},
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
            Err(Error::Internal(err)) if err.to_string().contains("UNIQUE constraint failed")
        ));
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
        assert!(!dir.exists());
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
