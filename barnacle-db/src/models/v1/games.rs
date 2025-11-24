use std::path::PathBuf;

use agdb::{DbId, DbSerialize, DbType, DbValue};
use strum::{Display, EnumIter};

use crate::GameId;

#[derive(
    Debug, Clone, Default, DbValue, DbSerialize, Copy, PartialEq, PartialOrd, Display, EnumIter,
)]
#[strum(serialize_all = "title_case")]
pub enum DeployKind {
    /// Deploys directly to the game directory with OverlayFS.
    #[default]
    Overlay,
    /// Same as the overlay type, but with support for Gamebryo/Creation Engine `plugins.txt`.
    Gamebryo,
    CreationEngine,
    /// Deploys mods to an intermediary staging directory with OverlayFS, preventing the mod store
    /// from needing to be modified. The individual mod directories are then added to `openmw.cfg`.
    /// Plugins are also handled.
    #[strum(serialize = "OpenMW")]
    OpenMW,
    #[strum(serialize = "Baldur's Gate 3")]
    BaldursGate3,
}

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Game {
    db_id: Option<DbId>,
    name: String,
    targets: Vec<PathBuf>,
    deploy_kind: DeployKind,
}

impl Game {
    pub fn new(name: &str, deploy_kind: DeployKind) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
            targets: Vec::new(),
            deploy_kind,
        }
    }

    pub fn id(&self) -> Option<GameId> {
        self.db_id.map(GameId)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn deploy_kind(&self) -> DeployKind {
        self.deploy_kind
    }
}
