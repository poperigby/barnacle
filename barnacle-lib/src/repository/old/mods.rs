use std::{fs::File, path::Path};

use barnacle_db::{GameId, ModId, models::Mod};
use compress_tools::{Ownership, uncompress_archive};

use crate::{
    Result,
    fs::{Permissions, change_dir_permissions},
    repository::Repository,
};

impl Repository {
    pub async fn add_mod(
        &mut self,
        game_id: GameId,
        input_path: Option<&Path>,
        name: &str,
    ) -> Result<ModId> {
        let new_mod = Mod::new(name);

        let game = self.db.read().await.game(game_id)?;
        let dir = self.cfg().await.mod_dir(&game, &new_mod);

        // TODO: Only attempt to open the archive if the input_path is an archive
        if let Some(path) = input_path {
            let archive = File::open(path)?;
            uncompress_archive(archive, &dir, Ownership::Preserve)?;
            change_dir_permissions(&dir, Permissions::ReadOnly);
        }

        Ok(self.db.write().await.insert_mod(&new_mod, game_id)?)
    }

    // pub fn delete_mod(db: &Database, id: ModId) -> Result<()> {
    //     db.remove_mod(id)?;
    //
    //     let dir = data_dir().join("mods").join(id.to_string());
    //
    //     change_dir_permissions(&dir, Permissions::ReadWrite);
    //     remove_dir_all(&dir)?;
    // }

    pub async fn mods(&self, game_id: GameId) -> Result<Vec<Mod>> {
        Ok(self.db.read().await.mods(game_id)?)
    }
}
