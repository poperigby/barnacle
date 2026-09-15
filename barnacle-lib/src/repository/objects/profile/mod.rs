use std::{
    fmt::Debug,
    fs::{self, create_dir_all},
    path::PathBuf,
};

mod children;
mod error;
mod fields;
mod lifecycle;

#[cfg(test)]
mod tests;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::info;

pub use error::*;

use crate::repository::{
    Cfg, Game, Mod, ModEntry,
    db::{
        Db,
        models::profiles::{ActiveModel, COLUMN, Entity, Model},
    },
    objects::{
        error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
        mod_entry,
    },
};

/// Represents a profile entity in the Barnacle system.
///
/// Provides methods to inspect and modify this profile's data, including
/// managing mod entries. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Profile {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Profile {
    pub(crate) fn from_id(id: i32, db: &Db, cfg: &Cfg) -> Self {
        Self {
            id,
            db: db.clone(),
            cfg: cfg.clone(),
        }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Profile, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Profile, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
