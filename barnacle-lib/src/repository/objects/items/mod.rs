use std::path::PathBuf;

use sea_orm::{EntityTrait, QueryFilter};

use crate::{
    error::{LoadModelError, ModelKind},
    repository::{
        config::Cfg,
        db::Db,
        items::{COLUMN, Entity, Model},
        mods::Mod,
    },
};

pub use kind::Kind;

pub mod kind;
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

    pub async fn parent(&self) -> Mod {
        let parent_id = self.model().await.unwrap().mod_id;

        Mod::from_id(parent_id, &self.db, &self.cfg)
    }

    pub async fn relative_path(&self) -> PathBuf {
        PathBuf::from(self.model().await.unwrap().relative_path)
    }

    /// The [`Item`]'s absolute path, nested within its parent [`Mod`] directory in the library.
    pub async fn absolute_path(&self) -> PathBuf {
        let parent_dir = self.parent().await.dir().await.unwrap();
        let relative_path = self.relative_path().await;

        parent_dir.join(relative_path)
    }

    pub async fn kind(&self) -> Kind {
        self.model().await.unwrap().kind
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, mod_: &Mod) -> Vec<Self> {
        Entity::find()
            .filter(COLUMN.mod_id.eq(mod_.id))
            .all(db.conn())
            .await
            .unwrap()
            .iter()
            .map(|m| Self::from_id(db, cfg, m.id))
            .collect()
    }
}
