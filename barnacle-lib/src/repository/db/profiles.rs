use std::fs::create_dir_all;

use barnacle_db::{GameId, ModId, ProfileId, ProfileMod, models::Profile};

use crate::{Result, repository::Repository};

impl Repository {
    pub async fn add_profile(&mut self, game_id: GameId, name: &str) -> Result<ProfileId> {
        let new_profile = Profile::new(name);

        let game = self.db.read().await.game(game_id)?;

        create_dir_all(self.cfg().await.core().profile_dir(&game, &new_profile))?;

        Ok(self
            .db
            .write()
            .await
            .insert_profile(&new_profile, game_id)?)
    }

    pub async fn current_profile(&self) -> Result<Option<ProfileId>> {
        Ok(self.db.read().await.current_profile()?)
    }

    pub async fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.db.write().await.set_current_profile(profile_id)?;

        Ok(())
    }

    pub async fn profiles(&self, game_id: GameId) -> Result<Vec<Profile>> {
        Ok(self.db.read().await.profiles(game_id)?)
    }

    pub async fn add_mod_entry(&mut self, mod_id: ModId, profile_id: ProfileId) -> Result<()> {
        self.db.write().await.insert_mod_entry(mod_id, profile_id)?;

        Ok(())
    }

    pub async fn profile_mods(&self, profile_id: ProfileId) -> Result<Vec<ProfileMod>> {
        Ok(self.db.read().await.profile_mods(profile_id)?)
    }
}
