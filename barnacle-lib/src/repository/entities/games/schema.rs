use sea_orm::entity::prelude::*;
use strum::{Display, EnumIter, EnumString};

use crate::entities;

#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, EnumString,
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

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    // Children
    #[sea_orm(has_many)]
    pub profiles: HasMany<entities::profiles::Entity>,

    #[sea_orm(has_many)]
    pub mods: HasMany<entities::mods::Entity>,

    #[sea_orm(has_many)]
    pub tools: HasMany<entities::tools::Entity>,

    #[sea_orm(unique)]
    pub name: String,
    pub targets: String,
    pub deploy_kind: String,
    pub is_active: bool,
}

impl ActiveModelBehavior for ActiveModel {}
