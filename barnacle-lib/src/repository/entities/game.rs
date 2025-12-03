use std::{
    fs::{self, File},
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
        entities::{Result, get_field, mod_::Mod, profile::Profile, set_field},
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
        if new_name == self.name()? {
            return Ok(());
        }

        let old_dir = self.dir()?;

        set_field(&mut self.db, self.id, "name", new_name)?;

        let new_dir = self.dir()?;
        fs::rename(old_dir, new_dir).unwrap();

        Ok(())
    }

    pub fn targets(&self) -> Result<Vec<PathBuf>> {
        get_field(&self.db, self.id, "targets")
    }

    pub fn deploy_kind(&self) -> Result<DeployKind> {
        get_field(&self.db, self.id, "deploy_kind")
    }

    pub fn set_deploy_kind(&mut self, new_deploy_kind: DeployKind) -> Result<()> {
        if new_deploy_kind == self.deploy_kind()? {
            return Ok(());
        }

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

        fs::create_dir_all(game.dir().unwrap()).unwrap();

        debug!("Created new game: {}", game.name()?);

        Ok(game)
    }

    pub(crate) fn remove(self) -> Result<()> {
        let name = self.name()?;
        let dir = self.dir()?;

        self.db
            .write()
            .exec_mut(QueryBuilder::remove().ids(self.id).query())?;

        fs::remove_dir_all(dir).unwrap();

        debug!("Removed game: {name}");

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
                .expect("A successful query should not be empty")
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

        fs::create_dir_all(profile.dir()?).unwrap();

        Ok(profile)
    }

    // TODO: Make this just a helper that calls Profile::remove()
    pub fn remove_profile(&mut self, profile: Profile) -> Result<()> {
        let name = profile.name()?;
        let dir = profile.dir()?;

        self.db
            .write()
            .exec_mut(QueryBuilder::remove().ids(profile.id).query())?;

        fs::remove_dir_all(dir).unwrap();

        debug!("Removed game: {name}");

        Ok(())
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
                .expect("A successful query should not be empty")
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
    use crate::Repository;

    use super::*;

    #[test]
    fn test_add() {
        let repo = Repository::mock();

        repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();

        let games = repo.games().unwrap();

        assert_eq!(games.len(), 2);
        assert_eq!(games.first().unwrap().name().unwrap(), "Morrowind");
        assert_eq!(
            games.last().unwrap().deploy_kind().unwrap(),
            DeployKind::CreationEngine
        );
    }

    #[test]
    fn test_remove() {
        let repo = Repository::mock();

        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        assert_eq!(repo.games().unwrap().len(), 1);

        repo.remove_game(game).unwrap();

        assert_eq!(repo.games().unwrap().len(), 0);
    }

    #[test]
    fn test_set_name() {
        let repo = Repository::mock();

        let mut game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        assert_eq!(game.name().unwrap(), "Skyrim");

        game.set_name("Skyrim 2: Electric Boogaloo").unwrap();

        assert_eq!(game.name().unwrap(), "Skyrim 2: Electric Boogaloo");
    }

    #[test]
    fn test_dir() {
        let repo = Repository::mock();

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .unwrap();

        let expected_dir = repo
            .cfg
            .read()
            .library_dir()
            .join(game.name().unwrap().to_snake_case());

        assert_eq!(game.dir().unwrap(), expected_dir);
    }
}
