use std::{future::Future, sync::Arc};

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use tokio::runtime::Runtime;

use crate::{
    fs::state_dir,
    repository::{
        db::models::{games, mod_entries, mods, profiles, tools},
        entities::Result,
    },
};

pub(crate) mod models;

#[derive(Clone, Debug)]
pub(crate) struct Db {
    conn: DatabaseConnection,
    runtime: Arc<Runtime>,
}

impl Db {
    pub fn new() -> Self {
        let runtime = Arc::new(Runtime::new().unwrap());
        let db_url = format!("sqlite:{}?mode=rwc", &state_dir().join("data.db").display());
        let conn = runtime
            .block_on(Database::connect(&db_url))
            .unwrap_or_else(|err| panic!("failed to open sqlite database: {err}"));

        let db = Self { conn, runtime };
        db.init().unwrap();
        db
    }

    fn init(&self) -> Result<()> {
        self.run(async {
            self.conn
                .execute_unprepared("PRAGMA foreign_keys = ON")
                .await?;
            self.conn
                .get_schema_builder()
                .register(games::Entity)
                .register(profiles::Entity)
                .register(mods::Entity)
                .register(mod_entries::Entity)
                .register(tools::Entity)
                .sync(&self.conn)
                .await?;

            Ok(())
        })
    }

    pub(crate) fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub(crate) fn run<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = std::result::Result<T, DbErr>>,
    {
        Ok(self.runtime.block_on(future)?)
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        let runtime = Arc::new(Runtime::new().unwrap());
        let conn = runtime
            .block_on(Database::connect("sqlite::memory:"))
            .unwrap();
        let db = Self { conn, runtime };
        db.init().unwrap();
        db
    }
}
