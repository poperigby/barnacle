use std::fs;

use barnacle_lib::fs::config_dir;
use serde::{Deserialize, Serialize};

use crate::config::theme::Theme;

mod theme;

const CURRENT_CONFIG_VERSION: u16 = 1;
const FILE_NAME: &str = "gui.toml";

/// The backend's core configuration, serialized to TOML.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    theme: Theme,
}

impl GuiConfig {
    pub fn load() -> Self {
        let path = config_dir().join(FILE_NAME);

        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            toml::from_str(&contents).unwrap_or_default()
        } else {
            let cfg = Self::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let contents = toml::to_string_pretty(self).unwrap();

        // Make sure config_dir exists
        fs::create_dir_all(config_dir()).unwrap();

        fs::write(config_dir().join(FILE_NAME), contents).unwrap();
    }

    pub fn theme(&self) -> iced::Theme {
        (&self.theme).into()
    }
}
