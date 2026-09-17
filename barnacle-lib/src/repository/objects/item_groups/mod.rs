use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};

use crate::{
    Game,
    deployers::ItemGroupSpec,
    repository::{
        config::Cfg,
        db::Db,
        objects::item_groups::model::{ActiveModel, COLUMN, Entity, Model},
    },
};

pub(crate) mod model;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct ItemGroup {
    pub id: i32,
    db: Db,
    cfg: Cfg,
}

impl ItemGroup {
    pub(crate) fn from_id(db: &Db, cfg: &Cfg, id: i32) -> Self {
        Self {
            db: db.clone(),
            cfg: cfg.clone(),
            id,
        }
    }

    async fn model(&self) -> Model {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await
            .unwrap()
            .unwrap()
    }

    pub(crate) async fn key(&self) -> String {
        self.model().await.key
    }

    pub(crate) async fn name(&self) -> String {
        self.model().await.name
    }

    pub(crate) async fn add(db: &Db, game: &Game, spec: impl ItemGroupSpec) {
        let model = ActiveModel {
            game_id: Set(game.id()),
            key: Set(spec.key().to_string()),
            name: Set(spec.name().to_string()),
            ordered: Set(spec.ordered()),
            ..Default::default()
        };

        Entity::insert(model).exec(db.conn()).await.unwrap();
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Vec<Self> {
        Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .unwrap()
            .iter()
            .map(|m| Self::from_id(db, cfg, m.id))
            .collect()
    }
}

impl PartialEq for ItemGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
