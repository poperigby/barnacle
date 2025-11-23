use std::fs::{self, create_dir_all};

use barnacle_lib::{config::CoreConfig, fs::config_dir};
use serde::{Deserialize, Serialize};

use crate::config::theme::Theme;

mod theme;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    theme: Theme,
}

impl GuiConfig {
    pub fn theme(&self) -> iced::Theme {
        (&self.theme).into()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    core: CoreConfig,
    gui: GuiConfig,
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
        create_dir_all(config_dir()).unwrap();

        fs::write(config_dir().join("config.toml"), contents).unwrap();
    }

    pub fn core(&self) -> &CoreConfig {
        &self.core
    }

    pub fn gui(&self) -> &GuiConfig {
        &self.gui
    }
}
