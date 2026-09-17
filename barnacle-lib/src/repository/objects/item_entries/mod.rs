use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter, QueryOrder};

mod error;
pub(crate) mod model;

pub use error::*;

use crate::{
    ModEntry, Profile,
    deployers::ItemGroup,
    error::{LoadModelError, ModelKind},
    repository::{
        config::Cfg,
        db::Db,
        item_entries::{ActiveModel, COLUMN, Entity, Model},
        item_lists, mod_entries,
        objects::{error::is_unique_violation, items::Item},
    },
};

pub struct ItemEntry {
    pub(crate) id: i32,
    db: Db,
    cfg: Cfg,
}

impl ItemEntry {
    pub(crate) fn from_id(id: i32, db: &Db, cfg: &Cfg) -> Self {
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
            .map_err(|source| LoadModelError::query(ModelKind::ItemEntry, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::ItemEntry, self.id))
    }

    async fn active_model(&self) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model().await?.into())
    }

    pub(crate) async fn add(
        db: &Db,
        cfg: &Cfg,
        mod_entry: &ModEntry,
        item: Item,
    ) -> Result<Self, AddError> {
        let conn = db.conn();

        let profile = mod_entry.parent().await.unwrap();
        let item_group = item.group().await;

        let list_id = item_lists::find(db, &profile, &item_group).await;

        let next_priority = Entity::find()
            .filter(COLUMN.item_list_id.eq(list_id))
            .order_by_desc(COLUMN.priority)
            .one(conn)
            .await
            .map_err(AddError::NextPriority)?
            .and_then(|e| e.priority.map(|priority| priority + 1))
            .unwrap_or(0);

        let model = ActiveModel {
            mod_entry_id: Set(mod_entry.id),
            item_id: Set(item.id),
            item_list_id: Set(list_id),
            priority: Set(Some(next_priority)),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(conn)
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    AddError::Duplicate
                } else {
                    AddError::Insert(source)
                }
            })?
            .last_insert_id;

        Ok(Self::from_id(id, db, cfg))
    }

    pub(crate) async fn list(
        db: &Db,
        cfg: &Cfg,
        profile: &Profile,
        group: impl ItemGroup,
    ) -> Vec<Self> {
        let conn = db.conn();

        let list_id = item_lists::find(db, profile, group.key()).await;

        let entries = Entity::find()
            .inner_join(mod_entries::Entity)
            .filter(COLUMN.item_list_id.eq(list_id))
            .filter(mod_entries::COLUMN.enabled.eq(true))
            .order_by_desc(COLUMN.priority)
            .all(conn)
            .await
            .unwrap();

        entries
            .iter()
            .map(|m| Self::from_id(m.id, db, cfg))
            .collect()
    }

    /// Returns the parent [`ModEntry`] of this [`ItemEntry`].
    pub async fn parent(&self) -> Result<ModEntry, ParentError> {
        Ok(ModEntry::from_id(
            self.model().await.map_err(ParentError::Load)?.mod_entry_id,
            &self.db,
            &self.cfg,
        ))
    }
}
