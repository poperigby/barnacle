use barnacle_db::Database;

use crate::{Result, fs::data_dir};

mod games;
mod mods;
mod profiles;
mod tools;

pub struct State {
    db: Database,
}

impl State {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Database::new(&data_dir().join("data.db"))?,
        })
    }
}
