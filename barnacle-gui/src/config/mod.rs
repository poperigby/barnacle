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
