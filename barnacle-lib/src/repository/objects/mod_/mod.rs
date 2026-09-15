use std::{
    fmt::Debug,
    fs::{self},
    path::PathBuf,
};

pub mod builder;
mod error;
mod fields;
mod lifecycle;

#[cfg(test)]
mod tests;

use sea_orm::{ConnectionTrait, EntityTrait, QueryFilter};
use tracing::info;

pub use error::*;

use crate::repository::{
    Cfg,
    db::{
        Db,
        models::mods::{COLUMN, Entity, Model},
    },
    objects::{
        error::{GetFieldError, LoadModelError, ModelKind},
        game::Game,
        mod_::builder::NewMod,
    },
};

/// Represents a mod entity in the Barnacle system.
///
/// Provides methods to inspect and modify this mod's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Mod {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Mod {
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
            .map_err(|source| LoadModelError::query(ModelKind::Mod, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Mod, self.id))
    }

}

impl PartialEq for Mod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
