use std::path::PathBuf;

use agdb::DbId;

use crate::repository::{
    db::{DbHandle, get_field},
    models::DeployKind,
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
pub struct Game {
    id: DbId,
    db: DbHandle,
}

impl Game {
    pub async fn name(&self) -> String {
        let db = self.db.read().await;

        get_field(&db, "name", self.id).unwrap()
    }

    pub async fn targets(&self) -> Vec<PathBuf> {
        let db = self.db.read().await;

        get_field(&db, "targets", self.id).unwrap()
    }

    pub async fn deploy_kind(&self) -> DeployKind {
        let db = self.db.read().await;

        get_field(&db, "deploy_kind", self.id).unwrap()
    }
}
