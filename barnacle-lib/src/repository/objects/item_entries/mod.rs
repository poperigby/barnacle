use std::path::PathBuf;

use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    error::{LoadModelError, ModelKind},
    repository::{
        Profile,
        config::Cfg,
        db::Db,
        item_entries::{ActiveModel, COLUMN, Entity, Model},
        items::{self, Item, kind::Kind},
    },
};

pub(crate) mod model;

#[derive(Debug, Clone)]
pub struct ItemEntry {
    pub id: i32,
    db: Db,
    cfg: Cfg,
}

impl ItemEntry {
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
            .map_err(|source| LoadModelError::query(ModelKind::ItemEntry, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::ItemEntry, self.id))
    }

    async fn item_model(&self) -> items::Model {
        let item_id = self.model().await.unwrap().item_id;

        items::Entity::find_by_id(item_id)
            .one(self.db.conn())
            .await
            .unwrap()
            .unwrap()
    }

    pub async fn enabled(&self) -> bool {
        self.model().await.unwrap().enabled
    }

    pub async fn relative_path(&self) -> PathBuf {
        PathBuf::from(self.item_model().await.relative_path)
    }

    pub(crate) async fn add(db: &Db, profile: &Profile, item: &Item) {
        let conn = db.conn();

        let next_position = Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .order_by_desc(COLUMN.position)
            .one(conn)
            .await
            .unwrap()
            .map_or(0, |e| e.position + 1);

        let model = ActiveModel {
            item_id: Set(item.id),
            profile_id: Set(profile.id),
            position: Set(next_position),
            ..Default::default()
        };

        Entity::insert(model).exec(conn).await.unwrap();
    }

    pub(crate) async fn list_by_kind(
        db: &Db,
        cfg: &Cfg,
        profile: &Profile,
        kind: Kind,
    ) -> Vec<ItemEntry> {
        Entity::find()
            .inner_join(items::Entity)
            .filter(COLUMN.profile_id.eq(profile.id))
            .filter(items::COLUMN.kind.eq(kind))
            .order_by_asc(COLUMN.position)
            .all(db.conn())
            .await
            .unwrap()
            .iter()
            .map(|m| Self::from_id(db, cfg, m.id))
            .collect()
    }

    pub(crate) async fn list_groups(
        db: &Db,
        cfg: &Cfg,
        profile: &Profile,
        kinds: &[Kind],
    ) -> Vec<ItemGroup> {
        let mut groups = Vec::with_capacity(kinds.len());

        for kind in kinds {
            let entries = Self::list_by_kind(db, cfg, profile, *kind).await;

            groups.push(ItemGroup {
                cfg: cfg.clone(),
                kind: *kind,
                entries,
            });
        }

        groups
    }
}

#[derive(Debug, Clone)]
pub struct ItemGroup {
    pub cfg: Cfg,
    pub kind: Kind,
    pub entries: Vec<ItemEntry>,
}

impl PartialEq for ItemGroup {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
