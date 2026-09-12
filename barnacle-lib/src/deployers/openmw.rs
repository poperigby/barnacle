use openmw_config::OpenMWConfiguration;

use crate::Game;

pub struct OpenMw {
    config: OpenMWConfiguration,
}

impl OpenMw {
    pub fn load() -> Self {
        let config = OpenMWConfiguration::from_env_or_user_config().unwrap();

        Self { config }
    }
    pub async fn deploy(&mut self, game: &Game) {
        let active_profile = game.active_profile().await.unwrap().unwrap();

        for mod_entry in active_profile.mod_entries().await.unwrap() {
            if mod_entry.enabled().await.unwrap() {
                self.config
                    .add_data_directory(&mod_entry.dir().await.unwrap());
            }
        }

        self.config.save_user().unwrap();
    }
}
