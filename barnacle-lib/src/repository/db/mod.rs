use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

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
}

impl Db {
    pub async fn new() -> Self {
        let db_url = format!("sqlite:{}?mode=rwc", &state_dir().join("data.db").display());
        let conn = Database::connect(&db_url)
            .await
            .unwrap_or_else(|err| panic!("failed to open sqlite database: {err}"));

        let db = Self { conn };
        db.init().await.unwrap();
        db
    }

    async fn init(&self) -> Result<()> {
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
    }

    pub(crate) fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    #[cfg(test)]
    pub(crate) async fn in_memory() -> Self {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let db = Self { conn };
        db.init().await.unwrap();
        db
    }
}
