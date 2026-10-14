use std::path::PathBuf;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryFilter, QueryOrder, raw_sql};

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

pub use group::ItemGroup;

mod group;
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

    async fn active_model(&self) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model().await?.into())
    }

    async fn item_model(&self) -> items::Model {
        let item_id = self.model().await.unwrap().item_id;

        items::Entity::find_by_id(item_id)
            .one(self.db.conn())
            .await
            .unwrap()
            .unwrap()
    }

    async fn item(&self) -> Item {
        let item_id = self.item_model().await.id;

        Item::from_id(&self.db, &self.cfg, item_id)
    }

    pub async fn enabled(&self) -> bool {
        self.model().await.unwrap().enabled
    }

    pub async fn set_enabled(&self, enabled: bool) {
        let mut active_model = self.active_model().await.unwrap();

        active_model.enabled.set_if_not_equals(enabled);

        active_model.update(self.db.conn()).await.unwrap();
    }

    pub async fn position(&self) -> i32 {
        self.model().await.unwrap().position
    }

    pub async fn set_position(&self, new_position: i32) {
        let mut active_model = self.active_model().await.unwrap();

        active_model.position.set_if_not_equals(new_position);

        active_model.update(self.db.conn()).await.unwrap();
    }

    pub async fn name(&self) -> String {
        let relative_path = self.relative_path().await;
        let file_name = relative_path
            .file_name()
            .expect("an Item must always have a valid file name");

        file_name
            .to_str()
            .expect("an Item's file name must be valid UTF-8")
            .to_string()
    }

    /// The [`ItemEntry`]'s path relative to the target path.
    pub async fn relative_path(&self) -> PathBuf {
        self.item().await.relative_path().await
    }

    /// The [`ItemEntry`]'s absolute path, within the library directory.
    pub async fn absolute_path(&self) -> PathBuf {
        self.item().await.absolute_path().await
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
        let profile_id = profile.id;
        let kind_key = kind.as_db_key();

        // Joins the respective items and mod entries to each item entry. They are then grouped by
        // relative_path and only the item entry who's mod entry wins the conflict is included.
        let query = raw_sql!(
            Sqlite,
            r#"
            -- We consider the ModEntry with highest position to be the conflict winner
            SELECT item_entry.*, MAX(mod_entry.position) AS winning_position
            FROM item_entries AS item_entry
            JOIN items AS item ON item.id = item_entry.item_id
            JOIN mod_entries AS mod_entry
                ON mod_entry.mod_id = item.mod_id
                AND mod_entry.profile_id = item_entry.profile_id
            WHERE item_entry.profile_id = {profile_id}
                AND item.kind = {kind_key}
                AND mod_entry.enabled = TRUE
            -- Group conflicting items, so we can choose the winner based on its ModEntry's position
            GROUP BY item.relative_path
            ORDER BY item_entry.position, item_entry.id
            "#
        );

        Entity::find()
            .from_raw_sql(query)
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
                profile: profile.clone(),
                kind: *kind,
                entries,
            });
        }

        groups
    }
}
