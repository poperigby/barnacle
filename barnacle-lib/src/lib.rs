use std::{
    fs::{File, create_dir_all},
    io,
    path::Path,
};

use barnacle_db::{
    Database,
    models::{Game, Mod, Profile},
};

use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;

use crate::fs::{Permissions, change_dir_permissions, data_dir, game_dir, mod_dir, profile_dir};

mod deployers;
mod fs;

pub use barnacle_db::{GameId, ModId, ProfileId, ProfileMod, models::DeployKind};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] compress_tools::Error),
    #[error("Database error: {0}")]
    Db(#[from] barnacle_db::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

pub struct State {
    db: Database,
}

impl State {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Database::new(&data_dir().join("data.db"))?,
        })
    }

    pub async fn add_game(&mut self, name: &str, game_type: DeployKind) -> Result<()> {
        let new_game = Game::new(name, game_type);

        create_dir_all(game_dir(&new_game))?;

        self.db.insert_game(&new_game).await?;

        Ok(())
    }

    pub async fn add_profile(&mut self, game_id: GameId, name: &str) -> Result<()> {
        let new_profile = Profile::new(name);

        let game = self.db.game(game_id).await?;

        create_dir_all(profile_dir(&game, &new_profile))?;

        self.db.insert_profile(&new_profile, game_id).await?;

        Ok(())
    }

    pub async fn current_profile(&self) -> Result<Option<ProfileId>> {
        Ok(self.db.current_profile().await?)
    }

    pub async fn add_mod(&mut self, game_id: GameId, input_path: &Path, name: &str) -> Result<()> {
        let new_mod = Mod::new(name);

        let game = self.db.game(game_id).await?;
        let dir = mod_dir(&game, &new_mod);

        // TODO: Only attempt to open the archive if the input_path is an archive
        let archive = File::open(input_path)?;
        uncompress_archive(archive, &dir, Ownership::Preserve)?;
        change_dir_permissions(&dir, Permissions::ReadOnly);

        self.db.insert_mod(&new_mod, game_id).await?;

        Ok(())
    }

    pub async fn mods(&self, profile_id: ProfileId) -> Result<Vec<ProfileMod>> {
        Ok(self.db.mods(profile_id).await?)
    }

    // pub fn delete_mod(db: &Database, id: ModId) -> Result<()> {
    //     db.remove_mod(id)?;
    //
    //     let dir = data_dir().join("mods").join(id.to_string());
    //
    //     change_dir_permissions(&dir, Permissions::ReadWrite);
    //     remove_dir_all(&dir)?;
    // }
}
