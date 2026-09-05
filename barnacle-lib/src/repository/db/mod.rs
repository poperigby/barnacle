use sea_orm::{Database, DatabaseConnection, DbErr};

use crate::fs::state_dir;

pub(crate) mod models;

#[derive(Debug, Clone)]
pub(crate) struct Db(DatabaseConnection);

impl Db {
    pub async fn new() -> Result<Self, DbErr> {
        let path = state_dir().join("data.db");
        let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());

        let connection = Self::connect(&url).await?;

        Ok(Self(connection))
    }

    /// Create a memory backed database for use in tests
    #[cfg(test)]
    pub(crate) async fn in_memory() -> Result<Self, DbErr> {
        let connection = Self::connect("sqlite::memory:").await?;

        Ok(Self(connection))
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.0
    }

    async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
        let connection = Database::connect(url).await.unwrap();

        connection
            .get_schema_registry(module_path!().split("::").next().unwrap())
            .sync(&connection)
            .await?;

        Ok(connection)
    }
}
