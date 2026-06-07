pub(crate) mod schema;

pub(crate) use schema::*;

use std::path::PathBuf;

use sea_orm::EntityTrait;

use crate::repository::{
    config::Cfg,
    db::Db,
    models::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct Tool {
    id: i64,
    db: Db,
}

impl Tool {
    #[allow(dead_code)]
    pub(crate) async fn load(row_id: i64, db: Db, _cfg: Cfg) -> Result<Self> {
        let model = Entity::find_by_id(row_id).one(db.conn()).await?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self { id: model.id, db })
    }

    async fn model(&self) -> Result<Model> {
        let model = Entity::find_by_id(self.id).one(self.db.conn()).await?;
        model.ok_or(Error::RemovedEntity)
    }

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(self.model().await?.path))
    }

    pub async fn args(&self) -> Result<String> {
        Ok(self.model().await?.args.unwrap_or_default())
    }
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
