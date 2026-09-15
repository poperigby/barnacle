use sea_orm::prelude::*;

use crate::repository::db::models::{items, profiles};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(default_value = true)]
    pub enabled: bool,
    #[sea_orm(unique_key = "item_entry_priority_per_profile")]
    pub priority: i32,

    pub item_id: i32,
    #[sea_orm(belongs_to, from = "item_id", to = "id", on_delete = "Cascade")]
    pub item: BelongsTo<items::Entity>,

    #[sea_orm(unique_key = "item_entry_priority_per_profile")]
    pub profile_id: i32,
    #[sea_orm(belongs_to, from = "profile_id", to = "id", on_delete = "Cascade")]
    pub profile: BelongsTo<profiles::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
