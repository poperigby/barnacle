use std::fs::create_dir_all;

use barnacle_db::{GameId, ModId, ProfileId, ProfileMod, models::Profile};

use crate::{Result, fs::profile_dir, state::State};

impl State {
    pub async fn add_profile(&mut self, game_id: GameId, name: &str) -> Result<ProfileId> {
        let new_profile = Profile::new(name);

        let game = self.db.game(game_id).await?;

        create_dir_all(profile_dir(&game, &new_profile))?;

        Ok(self.db.insert_profile(&new_profile, game_id).await?)
    }

    pub async fn current_profile(&self) -> Result<Option<ProfileId>> {
        Ok(self.db.current_profile().await?)
    }

    pub async fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.db.set_current_profile(profile_id).await?;

        Ok(())
    }

    pub async fn profiles(&self, game_id: GameId) -> Result<Vec<Profile>> {
        Ok(self.db.profiles(game_id).await?)
    }

    pub async fn add_mod_entry(&mut self, mod_id: ModId, profile_id: ProfileId) -> Result<()> {
        self.db.insert_mod_entry(mod_id, profile_id).await?;

        Ok(())
    }

    pub async fn profile_mods(&self, profile_id: ProfileId) -> Result<Vec<ProfileMod>> {
        Ok(self.db.profile_mods(profile_id).await?)
    }
}
