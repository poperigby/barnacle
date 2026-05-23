use sea_orm::entity::prelude::*;
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, EnumString,
)]
#[strum(serialize_all = "title_case")]
pub enum DeployKind {
    #[default]
    Overlay,
    Gamebryo,
    CreationEngine,
    #[strum(serialize = "OpenMW")]
    OpenMW,
    #[strum(serialize = "Baldur's Gate 3")]
    BaldursGate3,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
    pub targets_json: String,
    pub deploy_kind: String,
    pub is_active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::profiles::Entity")]
    Profiles,
    #[sea_orm(has_many = "super::mods::Entity")]
    Mods,
    #[sea_orm(has_many = "super::tools::Entity")]
    Tools,
}

impl ActiveModelBehavior for ActiveModel {}
