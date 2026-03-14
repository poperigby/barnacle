use std::path::PathBuf;

use sea_orm::EntityTrait;

use crate::repository::{
    config::Cfg,
    db::{Db, models::tools},
    entities::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct Tool {
    id: i64,
    db: Db,
    cfg: Cfg,
}

impl Tool {
    pub(crate) fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = db.run(tools::Entity::find_by_id(row_id).one(db.conn()))?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    fn model(&self) -> Result<tools::Model> {
        let model = self.db.run(tools::Entity::find_by_id(self.id).one(self.db.conn()))?;
        model.ok_or(Error::RemovedEntity)
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.model()?.name)
    }

    pub fn path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(self.model()?.path))
    }

    pub fn args(&self) -> Result<String> {
        Ok(self.model()?.args.unwrap_or_default())
    }
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
