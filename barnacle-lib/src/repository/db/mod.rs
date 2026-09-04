use derive_more::Deref;
use sea_orm::{Database, DatabaseConnection};

use crate::fs::state_dir;

pub(crate) mod models;

#[derive(Debug, Clone, Deref)]
pub(crate) struct Db {
    #[deref]
    conn: DatabaseConnection,
}

impl Db {
    pub async fn new() -> Self {
        let path = state_dir().join("data.db");
        let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());

        let connection = Self::connect(&url).await;

        Self { conn: connection }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    async fn connect(url: &str) -> DatabaseConnection {
        let connection = Database::connect(url).await.unwrap();

        connection
            .get_schema_registry(module_path!().split("::").next().unwrap())
            .sync(&connection)
            .await
            .unwrap();

        connection
    }

    /// Create a memory backed database for use in tests
    #[cfg(test)]
    pub(crate) async fn in_memory() -> Self {
        let connection = Self::connect("sqlite::memory:").await;

        Self { conn: connection }
    }
}
