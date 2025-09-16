use std::{
    fs::{File, create_dir_all},
    io,
    path::Path,
};

use barnacle_db::{
    Database, GameCtx,
    models::{DeployKind, Game, Mod, Profile},
};
use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;

use crate::fs::{Permissions, change_dir_permissions, game_dir, mod_dir, profile_dir};

mod deployers;
mod fs;

#[derive(Error, Debug)]
pub enum AddModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchive(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchive(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchive(#[from] compress_tools::Error),
}

pub async fn add_game(db: &mut Database, name: &str, game_type: DeployKind) {
    let new_game = Game::new(name, game_type);

    create_dir_all(game_dir(&new_game)).unwrap();

    db.insert_game(&new_game).await.unwrap();
}

pub async fn add_profile(db: &mut Database, game_ctx: GameCtx, name: &str) {
    let new_profile = Profile::new(name);

    let game = db.game(game_ctx).await.unwrap();

    create_dir_all(profile_dir(&game, &new_profile)).unwrap();

    db.insert_profile(&new_profile, game_ctx).await.unwrap();
}

pub async fn add_mod(
    db: &mut Database,
    game_ctx: GameCtx,
    input_path: &Path,
    name: &str,
) -> Result<(), AddModError> {
    let new_mod = Mod::new(name);

    let game = db.game(game_ctx).await.unwrap();
    let dir = mod_dir(&game, &new_mod);

    // TODO: Only do attempt to open the archive if the input_path is an archive
    let archive = File::open(input_path).map_err(AddModError::OpenArchive)?;
    uncompress_archive(archive, &dir, Ownership::Preserve)?;
    change_dir_permissions(&dir, Permissions::ReadOnly);

    db.insert_mod(&new_mod, game_ctx).await.unwrap();

    Ok(())
}

// pub fn delete_mod(db: &Database, id: ModId) {
//     db.remove_mod(id).unwrap();
//
//     let dir = data_dir().join("mods").join(id.to_string());
//
//     change_dir_permissions(&dir, Permissions::ReadWrite);
//     remove_dir_all(&dir).unwrap();
// }
