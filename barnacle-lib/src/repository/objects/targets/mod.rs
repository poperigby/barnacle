use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

mod error;
pub(crate) mod model;

use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter};

pub use error::*;
use tracing::info;

use crate::{
    Game,
    repository::{
        config::Cfg,
        db::Db,
        objects::error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
        targets::{ActiveModel, COLUMN, Entity, Model},
    },
};

#[derive(Debug, Clone)]
pub struct Target {
    id: i32,
    db: Db,
    cfg: Cfg,
}

impl Target {
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Target, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Target, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn path(&self) -> Result<PathBuf, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("path", source))?
            .path
            .into())
    }

    /// Returns the parent [`Target`] of this [`ModEntry`]
    pub async fn parent(&self) -> Result<Target, ParentError> {
        Ok(Target::from_id(
            self.model(self.db.conn())
                .await
                .map_err(ParentError::Load)?
                .game_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    pub(crate) async fn add(
        db: &Db,
        cfg: &Cfg,
        game: &Game,
        name: &str,
        path: &Path,
    ) -> Result<Self, AddError> {
        let path = path.to_str().ok_or(AddError::NotUtf8Path)?.to_owned();
        let name = name.to_string();
        let game_name = game.name().await.map_err(AddError::GameName)?;

        let model = ActiveModel {
            name: Set(name.clone()),
            path: Set(path),
            ..Default::default()
        };

        let id = match Entity::insert(model).exec(db.conn()).await {
            Ok(result) => result.last_insert_id,
            Err(source) if is_unique_violation(&source) => {
                return Err(AddError::DuplicateName {
                    game: game_name,
                    name,
                });
            }
            Err(source) => return Err(AddError::Insert(source)),
        };

        let target = Self::from_id(id, db.clone(), cfg.clone());

        info!("Added target '{}' to game '{}'", name, game_name);

        Ok(target)
    }

    /// Remove the given [`ModEntry`] from the list
    pub async fn remove(self) -> Result<(), RemoveError> {
        let game_name = self
            .parent()
            .await
            .map_err(RemoveError::Parent)?
            .name()
            .await
            .map_err(RemoveError::GameName)?;

        let target_name = self.name().await.map_err(RemoveError::TargetName)?;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        info!("Removed target '{}' from game '{}'", target_name, game_name);

        Ok(())
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Self::from_id(model.id, db.clone(), cfg.clone()))
            .collect())
    }
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
