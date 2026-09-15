use sea_orm::entity::prelude::*;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
#[strum(serialize_all = "title_case")]
pub enum DeployKind {
    Skyrim,
    FalloutNV,
    #[strum(serialize = "OpenMW")]
    OpenMW,
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

    /// Path to the game's main executable. Can be an absolute path or a command name.
    pub launch_program: String,
    #[sea_orm(default_value = "")]
    pub launch_args: String,

    pub active_profile_id: Option<i32>,
    #[sea_orm(
        belongs_to,
        relation_enum = "ActiveProfile",
        from = "active_profile_id",
        to = "id",
        on_delete = "SetNull"
    )]
    pub active_profile: BelongsTo<Option<super::profiles::Entity>>,

    #[sea_orm(has_many)]
    pub profiles: HasMany<super::profiles::Entity>,
    #[sea_orm(has_many)]
    pub mods: HasMany<super::mods::Entity>,
    #[sea_orm(has_many)]
    pub tools: HasMany<super::tools::Entity>,
    #[sea_orm(has_many)]
    pub targets: HasMany<super::targets::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
