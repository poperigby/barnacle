use sea_orm::{
    ActiveValue::Set, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};

pub(crate) mod model;

use crate::{
    ModEntry, Profile,
    deployers::ItemGroup,
    error::{LoadModelError, ModelKind},
    repository::{
        config::Cfg,
        db::Db,
        item::{
            self,
            entry::{ActiveModel, COLUMN, Entity, Model, Relation},
        },
        mod_entry,
        objects::{
            error::is_unique_violation,
            item::{AddError, ParentError, group},
        },
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
        item_id: i32,
    ) -> Result<Self, AddError> {
        let conn = db.conn();

        let next_priority = Entity::find()
            .order_by_desc(COLUMN.priority)
            .one(conn)
            .await
            .map_err(AddError::NextPriority)?
            .map_or(0, |e| e.priority + 1);

        let model = ActiveModel {
            mod_entry_id: Set(mod_entry.id),
            item_id: Set(item_id),
            priority: Set(next_priority),
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

        let group_id = group::find_id_by_key(db, &profile.parent().await.unwrap(), group).await;

        let entries = Entity::find()
            .join(JoinType::InnerJoin, Relation::ModEntry.def())
            .join(JoinType::InnerJoin, Relation::Item.def())
            .filter(mod_entry::COLUMN.profile_id.eq(profile.id))
            .filter(item::COLUMN.group_id.eq(group_id))
            .filter(mod_entry::COLUMN.enabled.eq(true))
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
