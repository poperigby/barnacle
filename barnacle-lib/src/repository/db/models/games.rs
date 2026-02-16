use std::path::PathBuf;

use agdb::{DbElement, DbId, DbSerialize, DbValue};
use sea_orm::entity::prelude::*;
use strum::{Display, EnumIter};

use crate::repository::entities::Uid;

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

#[derive(Debug, Clone, DbElement, PartialEq, PartialOrd)]
pub(crate) struct GameModel {
    db_id: Option<DbId>,
    uid: u64,
    name: String,
    targets: Vec<PathBuf>,
    deploy_kind: DeployKind,
}

impl GameModel {
    pub fn new(uid: Uid, name: &str, deploy_kind: DeployKind) -> Self {
        Self {
            db_id: None,
            uid: uid.0,
            name: name.to_string(),
            targets: Vec::new(),
            deploy_kind,
        }
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "game")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(has_many)]
    pub profiles: HasMany<super::profiles::Entity>,
    #[sea_orm(has_many)]
    pub mods: HasMany<super::mods::Entity>,

    #[sea_orm(unique)]
    pub name: String,
    pub targets: String,
    pub deploy_kind: String,
}

impl ActiveModelBehavior for ActiveModel {}
