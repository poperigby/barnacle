use crate::{Result, fs::data_dir};

mod games;
mod mods;
mod profiles;
mod tools;

#[derive(Clone, Debug)]
pub struct Database {
    db: barnacle_db::Database,
}

impl Database {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: barnacle_db::Database::new(&data_dir().join("data.db"))?,
        })
    }
}
