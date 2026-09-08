use std::{fmt::Debug, path::PathBuf};

mod error;

use sea_orm::{ConnectionTrait, EntityTrait};

pub use error::*;

use crate::repository::{
    config::Cfg,
    db::{
        Db,
        models::tools::{ActiveModel, Entity, Model},
    },
    handles::error::{GetFieldError, LoadModelError, ModelKind},
};

/// Represents a tool entity in the Barnacle system.
///
/// Provides methods to inspect and modify this tool's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Tool {
    id: i32,
    db: Db,
    cfg: Cfg,
}

impl Tool {
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Ok(Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Tool, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Tool, self.id))?)
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn path(&self) -> Result<PathBuf, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("path", source))?
            .path
            .into())
    }

    pub async fn args(&self) -> Result<Option<String>, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("args", source))?
            .args)
    }
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
