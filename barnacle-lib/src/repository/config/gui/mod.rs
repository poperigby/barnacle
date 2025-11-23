use serde::{Deserialize, Serialize};

use crate::repository::config::gui::theme::Theme;

mod theme;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    theme: Theme,
}
