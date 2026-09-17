use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};

use crate::{
    Mod,
    deployers::{ItemGroup, ParsedItem},
    error::{LoadModelError, ModelKind},
    repository::{
        config::Cfg,
        db::Db,
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

    pub async fn list(db: &Db, cfg: &Cfg, mod_: &Mod) -> Vec<Self> {
        Entity::find()
            .filter(COLUMN.mod_id.eq(mod_.id))
            .all(db.conn())
            .await
            .unwrap()
            .iter()
            .map(|m| Self::from_id(&db, &cfg, m.id))
            .collect()
    }

    pub async fn add_from_parsed<G: ItemGroup>(db: &Db, mod_: &Mod, parsed_item: ParsedItem<G>) {
        let model = ActiveModel {
            name: Set(parsed_item.name),
            group: Set(parsed_item.group.key().to_string()),
            relative_path: Set(parsed_item.relative_path),
            mod_id: Set(mod_.id),
            ..Default::default()
        };

        Entity::insert(model).exec(db.conn()).await.unwrap();
    }

    pub(crate) async fn group(&self) -> String {
        self.model().await.unwrap().group
    }
}
