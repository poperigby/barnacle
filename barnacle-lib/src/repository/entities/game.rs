use std::{
    fs::{File, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};

use agdb::{DbId, QueryBuilder, QueryId};
use compress_tools::{Ownership, uncompress_archive};
use heck::ToSnakeCase;
use tracing::debug;

use crate::{
    fs::{Permissions, change_dir_permissions},
    repository::{
        CoreConfigHandle,
        db::DbHandle,
        entities::{Error, Result, get_field, mod_::Mod, profile::Profile, set_field},
        models::{DeployKind, GameModel, ModModel, ProfileModel},
    },
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Game {
    id: DbId,
    db: DbHandle,
    cfg: CoreConfigHandle,
}

impl Game {
    pub(crate) fn from_id(id: DbId, db: DbHandle, cfg: CoreConfigHandle) -> Self {
        Self { id, db, cfg }
    }

    // TODO: Perform unique violation checking
    pub fn name(&self) -> Result<String> {
        get_field(&self.db, self.id, "name")
    }

    pub fn set_name(&mut self, new_name: &str) -> Result<()> {
        set_field(&mut self.db, self.id, "name", new_name)
    }

    pub fn targets(&self) -> Result<Vec<PathBuf>> {
        get_field(&self.db, self.id, "targets")
    }

    pub fn deploy_kind(&self) -> Result<DeployKind> {
        get_field(&self.db, self.id, "deploy_kind")
    }

    pub fn set_deploy_kind(&mut self, new_deploy_kind: DeployKind) -> Result<()> {
        set_field(&mut self.db, self.id, "deploy_kind", new_deploy_kind)
    }

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .cfg
            .read()
            .library_dir()
            .join(self.name()?.to_snake_case()))
    }

    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub(crate) fn add(db: DbHandle, cfg: CoreConfigHandle, model: GameModel) -> Result<Self> {
        if Game::list(db.clone(), cfg.clone())?
            .iter()
            .any(|g| g.name().unwrap() == model.name)
        {
            // return Err(Error::UniqueViolation(UniqueConstraint::GameName));
            panic!("UniqueViolation");
        }

        let game = db
            .write()
            .transaction_mut(|t| -> std::result::Result<Game, agdb::DbError> {
                let game_id = t
                    .exec_mut(QueryBuilder::insert().element(model).query())
                    .unwrap()
                    .elements
                    .first()
                    .unwrap()
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from("games")
                        .to(game_id)
                        .query(),
                )
                .unwrap();

                Ok(Game::from_id(game_id, db.clone(), cfg.clone()))
            })
            .unwrap();

        create_dir_all(game.dir().unwrap()).unwrap();

        debug!("Created new game: {}", game.name()?);

        Ok(game)
    }

    pub(crate) fn remove(&self) -> Result<()> {
        self.db
            .write()
            .exec_mut(QueryBuilder::remove().ids(self.id).query())?;

        remove_dir_all(self.dir()?).unwrap();

        Ok(())
    }

    pub(crate) fn list(db: DbHandle, cfg: CoreConfigHandle) -> Result<Vec<Game>> {
        Ok(db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<GameModel>()
                    .search()
                    .from("games")
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )?
            .elements
            .iter()
            .map(|e| Game::from_id(e.id, db.clone(), cfg.clone()))
            .collect())
    }

    pub fn add_profile(&mut self, name: &str) -> Result<Profile> {
        let model = ProfileModel::new(name);

        if self
            .profiles()?
            .iter()
            .any(|p: &Profile| p.name().unwrap() == model.name)
        {
            // return Err(Error::UniqueViolation(UniqueConstraint::ProfileName));
            panic!("Unique violation")
        }

        let profile = self.db.write().transaction_mut(|t| -> Result<Profile> {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(model).query())?
                .elements
                .first()
                .ok_or(Error::EmptyElements)?
                .id;

            // Link Profile to the specified Game node and root "profiles" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(self.id)])
                    .to(profile_id)
                    .query(),
            )?;

            Ok(Profile::from_id(
                profile_id,
                self.db.clone(),
                self.cfg.clone(),
            ))
        })?;

        create_dir_all(profile.dir()?).unwrap();

        Ok(profile)
    }

    pub fn profiles(&self) -> Result<Vec<Profile>> {
        Ok(self
            .db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<ProfileModel>()
                    .search()
                    .from(self.id)
                    .where_()
                    // .node()
                    // .and()
                    .neighbor()
                    .query(),
            )?
            .elements
            .iter()
            .map(|e| Profile::from_id(e.id, self.db.clone(), self.cfg.clone()))
            .collect())
    }

    pub fn add_mod(&mut self, name: &str, path: Option<&Path>) -> Result<Mod> {
        let new_mod = ModModel::new(name);

        // TODO: Only attempt to open the archive if the input_path is an archive
        if let Some(path) = path {
            let archive = File::open(path).unwrap();
            uncompress_archive(archive, &self.dir()?, Ownership::Preserve).unwrap();
            change_dir_permissions(&self.dir()?, Permissions::ReadOnly);
        }

        self.db.write().transaction_mut(|t| -> Result<Mod> {
            let mod_id = t
                .exec_mut(QueryBuilder::insert().element(new_mod).query())?
                .elements
                .first()
                .ok_or(Error::EmptyElements)?
                .id;

            // Link Profile to the specified Game node and root "profiles" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(self.id)])
                    .to(mod_id)
                    .query(),
            )?;

            Ok(Mod::from_id(mod_id, self.db.clone(), self.cfg.clone()))
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use parking_lot::RwLock;

    use crate::repository::config::CoreConfig;

    use super::*;

    #[test]
    fn test_add_game() {
        let db = DbHandle::new_memory();
        let cfg = Arc::new(RwLock::new(CoreConfig::default()));

        Game::add(
            db.clone(),
            cfg.clone(),
            GameModel::new("Skyrim", DeployKind::CreationEngine),
        )
        .unwrap();
        Game::add(
            db.clone(),
            cfg.clone(),
            GameModel::new("Morrowind", DeployKind::OpenMW),
        )
        .unwrap();

        let games = Game::list(db, cfg).unwrap();

        assert_eq!(games.len(), 2);
        assert_eq!(games.first().unwrap().name().unwrap(), "Morrowind");
        assert_eq!(
            games.last().unwrap().deploy_kind().unwrap(),
            DeployKind::CreationEngine
        );
    }
}
