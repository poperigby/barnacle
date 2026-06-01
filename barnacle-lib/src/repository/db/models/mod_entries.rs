use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mod_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub profile_id: i64,
    #[sea_orm(belongs_to, from = "profile_id", to = "id")]
    pub profile: HasOne<super::profiles::Entity>,

    pub mod_id: i64,
    #[sea_orm(belongs_to, from = "mod_id", to = "id")]
    #[sea_orm(column_name = "mod")]
    pub mod_: HasOne<super::mods::Entity>,

    #[sea_orm(unique_key = "profile_position")]
    pub position: i64,
    pub enabled: bool,
    pub notes: String,
}

impl ActiveModelBehavior for ActiveModel {}
