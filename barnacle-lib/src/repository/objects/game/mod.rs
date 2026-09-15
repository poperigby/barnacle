use std::{
    fmt::Debug,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

mod children;
mod error;
mod fields;
mod lifecycle;
mod launch;

#[cfg(test)]
mod tests;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, TransactionTrait};
use tokio::process::Command;
use tracing::info;

pub use error::*;

use crate::{
    deployers,
    fs::state_dir,
    mod_::{self, builder::NewMod},
    profile,
    repository::{
        Cfg, DeployKind,
        db::{
            Db,
            models::games::{ActiveModel, Entity, Model},
        },
        objects::{
            Target,
            error::{
                GetFieldError, LoadModelError, ModelKind, is_unique_violation,
                map_transaction_error,
            },
            mod_::Mod,
            profile::Profile,
            target,
        },
        state,
    },
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Game {
    id: i32,
    db: Db,
    cfg: Cfg,
}

impl Game {
    /// Load some existing [`Game`] from the database
    pub(crate) fn from_id(id: i32, db: &Db, cfg: &Cfg) -> Self {
        Self {
            id,
            db: db.clone(),
            cfg: cfg.clone(),
        }
    }

    pub(crate) fn id(&self) -> i32 {
        self.id
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Game, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Game, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
