use sea_orm::prelude::*;

use crate::repository::db::models::{games, items};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique_key = "item_group_name_per_game")]
    pub name: String,

    #[sea_orm(unique_key = "item_group_name_per_game")]
    pub game_id: i32,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: BelongsTo<games::Entity>,

    #[sea_orm(has_many)]
    pub items: HasMany<items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
