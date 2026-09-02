use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum DeployKind {
    /// Deploys directly to the game directory with an overlay filesystem.
    Overlay,
    /// Same as the overlay type, but with support for Gamebryo/Creation Engine `plugins.txt`.
    Gamebryo,
    CreationEngine,
    /// Deploys mods to an intermediary staging directory with OverlayFS, preventing the mod store
    /// from needing to be modified. The individual mod directories are then added to `openmw.cfg`.
    /// Plugins are also handled.
    OpenMW,
    BaldursGate3,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub name: String,
    pub deploy_kind: DeployKind,

    #[sea_orm(has_many)]
    pub profiles: HasMany<super::profiles::Entity>,
    #[sea_orm(has_many)]
    pub mods: HasMany<super::mods::Entity>,
    #[sea_orm(has_many)]
    pub tools: HasMany<super::tools::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
