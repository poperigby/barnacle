use openmw_config::OpenMWConfiguration;

use crate::Game;

pub struct OpenMw {
    config: OpenMWConfiguration,
}

impl OpenMw {
    pub fn load() -> Self {
        let config = OpenMWConfiguration::from_env().unwrap();

        Self { config }
    }
    pub async fn deploy(&self, game: &Game) {
        let active_profile = game.active_profile().await.unwrap().unwrap();

        for mod_entry in active_profile.mod_entries().await.unwrap() {}
    }
}
