use std::{fs, sync::Arc};

use barnacle_lib::fs::config_dir;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::persistence::config::theme::Theme;

mod theme;

const CURRENT_CONFIG_VERSION: u16 = 1;
const FILE_NAME: &str = "gui.toml";

#[derive(Debug, Clone)]
pub struct ConfigStore {
    inner: Arc<RwLock<Config>>,
}

impl ConfigStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Config::load())),
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::from(&self.inner.read().theme)
    }

    pub fn set_theme(&mut self, iced_theme: iced::Theme) {
        self.inner.write().theme = Theme::from(iced_theme);
        self.save();
    }

    fn save(&mut self) {
        self.inner.write().save();
    }
}

/// The GUI configuration, serialized to TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    pub version: u16,
    pub theme: Theme,
}

impl Config {
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

        fs::write(config_dir().join(FILE_NAME), contents).unwrap();
    }

    pub fn theme(&self) -> iced::Theme {
        (&self.theme).into()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            theme: Theme::default(),
        }
    }
}
