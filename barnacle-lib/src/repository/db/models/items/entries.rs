use sea_orm::prelude::*;

use crate::repository::db::models::items;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(default_value = true)]
    pub enabled: bool,

    pub item_id: i32,
    #[sea_orm(belongs_to, from = "item_id", to = "id", on_delete = "Cascade")]
    pub item: BelongsTo<items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
