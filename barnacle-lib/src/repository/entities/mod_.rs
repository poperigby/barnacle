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
        db::{
            Db,
            models::mods,
        },
        entities::{Error, Game, Result, map_duplicate_name},
    },
};

#[derive(Debug, Clone)]
pub struct Mod {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Mod {
    pub(crate) fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = db.run(mods::Entity::find_by_id(row_id).one(db.conn()))?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    fn model(&self) -> Result<mods::Model> {
        let model = self.db.run(mods::Entity::find_by_id(self.id).one(self.db.conn()))?;
        model.ok_or(Error::RemovedEntity)
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.model()?.name)
    }

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .parent()?
            .dir()?
            .join("mods")
            .join(self.name()?.to_snake_case()))
    }

    pub fn parent(&self) -> Result<Game> {
        let game_id = self.model()?.game_id;
        Game::load(game_id, self.db.clone(), self.cfg.clone())
    }

    pub(crate) fn add(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
        path: Option<&Path>,
    ) -> Result<Self> {
        let inserted = db.run(async {
            let model = mods::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                game_id: Set(game.id),
                name: Set(name.to_string()),
            };
            model.insert(db.conn()).await
        })
        .map_err(map_duplicate_name)?;

        let mod_ = Mod::load(inserted.id, db.clone(), cfg.clone())?;

        if let Some(path) = path {
            let archive = File::open(path).unwrap();
            uncompress_archive(archive, &mod_.dir()?, Ownership::Preserve).unwrap();
            change_dir_permissions(&mod_.dir()?, Permissions::ReadOnly);
        } else {
            fs::create_dir_all(mod_.dir()?).unwrap();
        }

        Ok(mod_)
    }

    pub fn remove(self) -> Result<()> {
        let name = self.name()?;
        let dir = self.dir()?;
        self.db.run(async {
            let Some(model) = mods::Entity::find_by_id(self.id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing mod during delete".into()));
            };
            model.delete(self.db.conn()).await?;
            Ok(())
        })?;

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

    #[test]
    fn test_add() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let mod_ = game.add_mod("Test", None).unwrap();

        assert!(mod_.dir().unwrap().exists());
    }

    #[test]
    fn test_add_duplicate() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        game.add_mod("Test", None).unwrap();

        assert!(matches!(
            game.add_mod("Test", None),
            Err(Error::DuplicateName)
        ));
    }

    #[test]
    fn test_remove() {
        let repo = Repository::mock();

        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let mod_ = game.add_mod("Test", None).unwrap();

        assert_eq!(game.mods().unwrap().len(), 1);

        let dir = mod_.dir().unwrap();

        mod_.remove().unwrap();

        assert_eq!(game.mods().unwrap().len(), 0);
        assert!(!dir.exists());
    }

    #[test]
    fn test_list() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        assert_eq!(game.mods().unwrap().len(), 0);

        game.add_mod("Better Spoon Textures 8K", None).unwrap();

        assert_eq!(game.mods().unwrap().len(), 1);
    }

    #[test]
    fn test_parent() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let mod_ = game.add_mod("Test", None).unwrap();

        assert_eq!(mod_.parent().unwrap(), game);
    }

    #[test]
    fn test_name() {
        let repo = Repository::mock();

        repo.add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .unwrap()
            .add_mod("Test", None)
            .unwrap()
            .name()
            .unwrap();
    }
}
