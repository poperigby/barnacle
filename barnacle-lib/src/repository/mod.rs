use std::sync::Arc;

use agdb::QueryBuilder;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{
    Result,
    repository::{
        config::CoreConfig,
        db::DbHandle,
        entities::{game::Game, profile::Profile},
        models::{DeployKind, GameModel},
    },
};

pub mod config;
pub mod db;
pub mod entities;
mod models;

pub type CoreConfigHandle = Arc<RwLock<CoreConfig>>;

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database and configuration file queries. It provides a single, consistent interface
/// for reading and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: DbHandle,
    cfg: CoreConfigHandle,
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: DbHandle::new(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        })
    }

    pub async fn cfg(&'_ self) -> RwLockReadGuard<'_, CoreConfig> {
        self.cfg.read()
    }

    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub fn add_game(&mut self, name: &str, deploy_kind: DeployKind) -> Game {
        let new_game_model = GameModel::new(name, deploy_kind);

        if self.games().iter().any(|g| g.name() == new_game_model.name) {
            // return Err(Error::UniqueViolation(UniqueConstraint::GameName));
            panic!("UniqueViolation");
        }

        self.db
            .write()
            .transaction_mut(|t| -> std::result::Result<Game, agdb::DbError> {
                let game_id = t
                    .exec_mut(QueryBuilder::insert().element(new_game_model).query())
                    .unwrap()
                    .elements
                    .first()
                    .unwrap()
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from("games")
                        .to(game_id)
                        .query(),
                )
                .unwrap();

                Ok(Game::from_id(game_id, self.db.clone(), self.cfg.clone()))
            })
            .unwrap()
    }

    pub fn games(&self) -> Vec<Game> {
        self.db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<GameModel>()
                    .search()
                    .from("games")
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )
            .unwrap()
            .elements
            .iter()
            .map(|e| Game::from_id(e.id, self.db.clone(), self.cfg.clone()))
            .collect()
    }
}
