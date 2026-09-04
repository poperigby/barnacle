use std::{fmt::Debug, path::PathBuf};

use sea_orm::EntityTrait;

use crate::repository::{
    config::Cfg,
    db::{
        Db,
        models::tools::{ActiveModel, Entity, Model},
    },
    handles::{Error, Result},
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

    async fn model(&self) -> Result<Model> {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    async fn active_model(&self) -> Result<ActiveModel> {
        Ok(self.model().await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn path(&self) -> Result<PathBuf> {
        Ok(self.model().await?.path.into())
    }

    pub async fn args(&self) -> Result<Option<String>> {
        Ok(self.model().await?.args)
    }
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
