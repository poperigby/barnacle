use std::{fs, sync::Arc};

use barnacle_lib::fs::state_dir;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::workspace::mod_list::state::SortState;

const FILE_NAME: &str = "gui.toml";

/// Persisted GUI state, such as selected tab, sort order, etc.
#[derive(Debug, Clone)]
pub struct UiStateStore {
    inner: Arc<RwLock<UiState>>,
}

impl UiStateStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(UiState::load())),
        }
    }

    pub fn mod_list_sort_state(&self) -> SortState {
        self.inner.read().mod_list.sort_state
    }

    pub fn set_mod_list_sort_state(&mut self, new_sort_state: SortState) {
        self.inner.write().mod_list.sort_state = new_sort_state;
        self.save();
    }

    fn save(&mut self) {
        self.inner.write().save();
    }
}

/// The GUI state, serialized to TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiState {
    pub mod_list: ModList,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ModList {
    pub sort_state: SortState,
}

impl UiState {
    pub fn load() -> Self {
        let path = state_dir().join(FILE_NAME);

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

        fs::write(state_dir().join(FILE_NAME), contents).unwrap();
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mod_list: ModList::default(),
        }
    }
}
