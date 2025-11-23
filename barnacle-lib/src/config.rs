use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::fs::data_dir;

const CURRENT_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreConfig {
    version: u32,
    data_dir: PathBuf,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            data_dir: data_dir(),
        }
    }
}
