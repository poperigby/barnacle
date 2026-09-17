use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};

use crate::{
    Mod,
    deployers::{DiscoveredItem, ItemGroupSpec},
    error::{LoadModelError, ModelKind},
    repository::{
        config::Cfg,
        db::Db,
        item_groups::ItemGroup,
        items::{ActiveModel, COLUMN, Entity, Model},
    },
};

pub(crate) mod model;

pub struct Item {
    pub(crate) id: i32,
    db: Db,
    cfg: Cfg,
}

impl Item {
    pub(crate) fn from_id(db: &Db, cfg: &Cfg, id: i32) -> Self {
        Self {
            id,
            db: db.clone(),
            cfg: cfg.clone(),
        }
    }

    async fn model(&self) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Item, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Item, self.id))
    }

    pub async fn list(db: &Db, cfg: &Cfg, group: &ItemGroup) -> Vec<Self> {
        Entity::find()
            .filter(COLUMN.group_id.eq(group.id))
            .all(db.conn())
            .await
            .unwrap()
            .iter()
            .map(|m| Self::from_id(&db, &cfg, m.id))
            .collect()
    }
}
