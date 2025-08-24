use agdb::{DbId, DbSerialize, DbType, DbValue};

#[derive(Debug, Clone, Default, DbValue, DbSerialize)]
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
    OpenMW,
}

#[derive(Debug, Clone, DbType)]
pub struct Game {
    db_id: Option<DbId>,
    name: String,
    deploy_kind: DeployKind,
}

impl Game {
    pub fn new(name: &str, deploy_kind: DeployKind) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
            deploy_kind,
        }
    }
}
