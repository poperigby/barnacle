use std::path::Path;

use agdb::{Db, DbError, DbId, QueryBuilder};
use thiserror::Error;

use crate::schema::v1::{
    mods::{Mod, ModEntry},
    profiles::Profile,
};

pub mod games;
pub mod mods;
pub mod profiles;

type Result<T> = std::result::Result<T, DatabaseError>;

/// ID representing a Game in the database
#[derive(Debug)]
pub struct GameId(DbId);

/// ID representing a Profile in the database
#[derive(Debug)]
pub struct ProfileId(DbId);

/// ID representing a Mod in the database
#[derive(Debug)]
pub struct ModId(DbId);

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(Db);

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    #[error("The given path is invalid unicode")]
    PathInvalidUnicode,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or(DatabaseError::PathInvalidUnicode)?;
        let db = Db::new(path_str)?;
        Ok(Database(db))
    }

    /// Initialize the root nodes
    pub fn init(&mut self) -> Result<()> {
        self.0.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["games", "current_profile"])
                .query(),
        )?;

        Ok(())
    }

    /// Insert a new Profile, linked to the given Game node
    pub fn insert_profile(&mut self, profile: &Profile, game_id: GameId) -> Result<ProfileId> {
        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(profile).query())?
                .elements[0]
                .id;

            // Link Profile to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id.0)
                    .to(profile_id)
                    .query(),
            )?;

            Ok(ProfileId(profile_id))
        })
    }

    pub fn current_profile(&self) -> Result<Profile> {
        let profile: Profile = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from("current_profile")
                    .query(),
            )?
            .try_into()?;

        Ok(profile)
    }

    pub fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // Delete existing current_profile, if it exists
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .from("current_profile")
                    .where_()
                    .edge()
                    .query(),
            )?;
            // Insert a new edge from current_profile to new profile_id
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from("current_profile")
                    .to(profile_id.0)
                    .query(),
            )?;

            Ok(())
        })
    }

    /// Add a new ModEntry to a Profile that points to a Mod
    pub fn link_mod_to_profile(&mut self, mod_id: ModId, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // We don't have to worry about adding mods from the wrong game, because you can only query
            // a Mod from a Game node.
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements[0]
                .id;

            // Insert ModEntry in between Profile and Mod
            // Profile -> ModEntry -> Mod
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(profile_id.0)
                    .to(mod_entry_id)
                    .query(),
            )?;
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(mod_entry_id)
                    .to(mod_id.0)
                    .query(),
            )?;

            Ok(())
        })
    }

    /// Insert a new Mod, linked to the given Game node
    pub fn insert_mod(&mut self, new_mod: &Mod, game_id: GameId) -> Result<ModId> {
        self.0.transaction_mut(|t| {
            let mod_id = t
                .exec_mut(QueryBuilder::insert().element(new_mod).query())?
                .elements[0]
                .id;

            // Link Mod to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id.0)
                    .to(mod_id)
                    .query(),
            )?;
            Ok(ModId(mod_id))
        })
    }
}
