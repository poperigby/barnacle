use std::{fmt::Debug, path::PathBuf};

mod error;
mod fields;
mod lifecycle;
mod relations;

#[cfg(test)]
mod tests;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
};
use tracing::info;

pub use error::*;

use crate::repository::{
    Mod, Profile,
    config::Cfg,
    db::{
        Db,
        models::mods::entries::{ActiveModel, COLUMN, Entity, Model},
    },
    objects::error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
};

/// Represents a mod entry in the Barnacle system.
///
/// Provides methods to inspect and modify this mod entry's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct ModEntry {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl ModEntry {
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::ModEntry, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::ModEntry, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

}

impl PartialEq for ModEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
