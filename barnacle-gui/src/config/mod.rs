use std::{fs, sync::Arc};

use barnacle_lib::fs::config_dir;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{app::mod_list::state::SortState, config::theme::Theme};

mod theme;

const CURRENT_CONFIG_VERSION: u16 = 1;
const FILE_NAME: &str = "gui.toml";

/// Handle to backend's core configuration
pub type Cfg = Arc<RwLock<GuiConfig>>;

/// The backend's core configuration, serialized to TOML.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub theme: Theme,
    pub mod_list: ModList,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModList {
    pub sort_state: SortState,
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
