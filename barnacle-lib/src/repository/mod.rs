use std::sync::Arc;

use barnacle_l10n::{TranslationBundle, Translations};
use parking_lot::RwLock;

use crate::repository::{
    config::{Cfg, CoreConfig},
    db::Db,
};

mod db;
mod state;

pub mod config;
pub mod objects;

pub mod games {
    pub use super::objects::games::model::Kind;
    pub(crate) use super::objects::games::model::*;
    pub use super::objects::games::*;
}

pub mod item_entries {
    pub(crate) use super::objects::item_entries::model::*;
    pub use super::objects::item_entries::*;
}

pub mod items {
    pub(crate) use super::objects::items::model::*;
    pub use super::objects::items::*;
}

pub mod mods {
    pub(crate) use super::objects::mods::model::*;
    pub use super::objects::mods::*;
}

pub mod mod_entries {
    pub(crate) use super::objects::mod_entries::model::*;
    pub use super::objects::mod_entries::*;
}

pub mod profiles {
    pub(crate) use super::objects::profiles::model::*;
    pub use super::objects::profiles::*;
}

pub mod targets {
    pub(crate) use super::objects::targets::model::*;
    pub use super::objects::targets::*;
}

pub mod tools {
    pub(crate) use super::objects::tools::model::*;
    pub use super::objects::tools::*;
}

pub use objects::{Game, Mod, ModEntry, Profile, Tool};

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database and configuration file queries. It provides a single, consistent interface
/// for reading and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: Db,
    cfg: Cfg,
}

impl Repository {
    pub async fn new() -> Self {
        Self {
            // TODO: WTF? Make this return a Result
            db: Db::new().await.unwrap(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        }
    }

    pub async fn translations(&self) -> Result<Translations, barnacle_l10n::TranslationsError> {
        let cfg_locale = self.cfg.read().locale().map(|l| l.id);
        barnacle_l10n::translations(cfg_locale)
    }

    pub async fn add_game(
        &self,
        name: &str,
        kind: games::Kind,
    ) -> Result<Game, objects::games::AddError> {
        Game::add(&self.db, &self.cfg, name, kind).await
    }

    pub async fn games(&self) -> Result<Vec<Game>, objects::games::ListError> {
        Game::list(self.db.clone(), self.cfg.clone()).await
    }

    pub async fn search_game(
        &self,
        name: &str,
    ) -> Result<Option<Game>, objects::games::SearchError> {
        Game::search(self.db.clone(), self.cfg.clone(), name).await
    }

    pub async fn active_game(&self) -> Result<Option<Game>, objects::games::ActiveError> {
        Game::active(&self.db, &self.cfg).await
    }

    /// An in-memory version of a [`Repository`] with a database and configuration
    /// file, for use in tests.
    #[cfg(test)]
    pub(crate) async fn in_memory() -> Self {
        Self {
            db: Db::in_memory().await.unwrap(),
            cfg: Arc::new(RwLock::new(CoreConfig::mock())),
        }
    }
}
