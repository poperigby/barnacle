use sea_orm::prelude::*;

use crate::repository::{item, mod_entry};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(default_value = true)]
    pub enabled: bool,
    pub priority: i32,

    #[sea_orm(unique_key = "item_id_per_mod_entry")]
    pub item_id: i32,
    #[sea_orm(belongs_to, from = "item_id", to = "id", on_delete = "Cascade")]
    pub item: BelongsTo<item::Entity>,

    #[sea_orm(unique_key = "item_id_per_mod_entry")]
    pub mod_entry_id: i32,
    #[sea_orm(belongs_to, from = "mod_entry_id", to = "id", on_delete = "Cascade")]
    pub mod_entry: BelongsTo<mod_entry::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
