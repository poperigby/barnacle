use sea_orm::{Database, DatabaseConnection};

use crate::{fs::state_dir, repository::models::Result};

#[derive(Clone, Debug)]
pub(crate) struct Db {
    db: DatabaseConnection,
}

impl Db {
    pub async fn new() -> Self {
        let db_url = format!("sqlite:{}?mode=rwc", &state_dir().join("data.db").display());
        let conn = Database::connect(&db_url)
            .await
            .unwrap_or_else(|err| panic!("failed to open sqlite database: {err}"));

        let db = Self { db: conn };
        db.init().await.unwrap();
        db
    }

    async fn init(&self) -> Result<()> {
        let model_path = module_path!().split("::").next().unwrap();
        self.db
            .get_schema_registry(model_path)
            .sync(&self.db)
            .await?;

        Ok(())
    }

    pub(crate) fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    #[cfg(test)]
    pub(crate) async fn in_memory() -> Self {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let db = Self { db: conn };
        db.init().await.unwrap();
        db
    }
}
