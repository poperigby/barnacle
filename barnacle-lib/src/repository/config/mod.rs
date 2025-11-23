use std::fs;

use serde::{Deserialize, Serialize};

use crate::{fs::config_dir, repository::config::core::CoreConfig};

mod core;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    core: CoreConfig,
}

impl Config {
    pub fn load() -> Self {
        let path = config_dir().join("config.toml");

        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            toml::from_str(&contents).unwrap_or_default()
        } else {
            let cfg = Config::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let contents = toml::to_string_pretty(self).unwrap();

        // Make sure config_dir exists
        fs::create_dir_all(config_dir()).unwrap();

        fs::write(config_dir().join("config.toml"), contents).unwrap();
    }

    pub fn core(&self) -> &CoreConfig {
        &self.core
    }
}
