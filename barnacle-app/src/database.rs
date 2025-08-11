#![allow(clippy::result_large_err)]
use barnacle_data::v1::{
    games::{Game, GameId},
    mods::{Mod, ModId},
    profiles::{Profile, ProfileId},
};
use thiserror::Error;

type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Native(#[from] native_db::db_type::Error),
    #[error("Failed to find game with ID: {0}")]
    GameNotFound(GameId),
    #[error("Failed to find profile with ID: {0}")]
    ProfileNotFound(ProfileId),
    #[error("Failed to find mod with ID: {0}")]
    ModNotFound(ModId),
}

/// Client for performing operations on the database.
/// The database is responsible for storing pure domain data.
pub struct Database<'a> {
    db: native_db::Database<'a>,
}

impl<'a> Database<'a> {
    pub fn new(db: native_db::Database<'a>) -> Self {
        Self { db }
    }

    pub fn insert_game(&self, new_game: Game) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        rw.insert(new_game)?;
        rw.commit()?;

        Ok(())
    }

    pub fn get_game(&self, id: GameId) -> Result<Option<Game>> {
        let r = self.db.r_transaction()?;
        Ok(r.get().primary(id)?)
    }

    pub fn remove_game(&self, id: GameId) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        let found_game: Game = rw
            .get()
            .primary(id)?
            .ok_or(DatabaseError::GameNotFound(id))?;

        rw.remove(found_game)?;
        rw.commit()?;

        Ok(())
    }

    pub fn insert_profile(&self, new_profile: Profile) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        rw.insert(new_profile)?;
        rw.commit()?;

        Ok(())
    }

    pub fn remove_profile(&self, id: ProfileId) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        let found_profile: Profile = rw
            .get()
            .primary(id)?
            .ok_or(DatabaseError::ProfileNotFound(id))?;

        rw.remove(found_profile)?;
        rw.commit()?;

        Ok(())
    }

    pub fn insert_mod(&self, new_mod: Mod) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        rw.insert(new_mod)?;
        rw.commit()?;

        Ok(())
    }

    pub fn get_mod(&self, id: ModId) -> Result<Option<Mod>> {
        let r = self.db.r_transaction()?;
        Ok(r.get().primary(id)?)
    }

    pub fn remove_mod(&self, id: ModId) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        let found_mod: Mod = rw
            .get()
            .primary(id)?
            .ok_or(DatabaseError::ModNotFound(id))?;

        rw.remove(found_mod)?;
        rw.commit()?;

        Ok(())
    }
}
