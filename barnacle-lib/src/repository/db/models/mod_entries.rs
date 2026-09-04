use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mod_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(default_value = true)]
    pub enabled: bool,
    #[sea_orm(unique_key = "mod_entry_priority_per_profile")]
    pub priority: i32,
    #[sea_orm(default_value = "")]
    pub notes: String,

    #[sea_orm(
        unique_key = "mod_entry_per_profile",
        unique_key = "mod_entry_priority_per_profile"
    )]
    pub profile_id: i32,
    #[sea_orm(belongs_to, from = "profile_id", to = "id", on_delete = "Cascade")]
    pub profile: BelongsTo<super::profiles::Entity>,

    #[sea_orm(unique_key = "mod_entry_per_profile")]
    pub mod_id: i32,
    #[sea_orm(belongs_to, from = "mod_id", to = "id", on_delete = "Cascade")]
    pub mod_: BelongsTo<super::mods::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
