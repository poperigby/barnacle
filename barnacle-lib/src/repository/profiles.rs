use agdb::DbId;

use crate::repository::db::{DbHandle, get_field};

/// Represents a profile entity in the Barnacle system.
///
/// Provides methods to inspect and modify this profile's data, including
/// managing mod entries. Always reflects the current database state.
pub struct Profile {
    id: DbId,
    db: DbHandle,
}

impl Profile {
    pub async fn name(&self) -> String {
        let db = self.db.read().await;

        get_field(&db, "name", self.id).unwrap()
    }
}
