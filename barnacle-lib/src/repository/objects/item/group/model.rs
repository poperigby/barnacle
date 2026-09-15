use sea_orm::prelude::*;

use crate::repository::{game, item};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique_key = "name_per_game")]
    pub name: String,

    #[sea_orm(unique_key = "key_per_game")]
    pub key: String,

    pub sortable: bool,

    #[sea_orm(unique_key = "name_per_game")]
    #[sea_orm(unique_key = "key_per_game")]
    pub game_id: i32,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: BelongsTo<game::Entity>,

    #[sea_orm(has_many)]
    pub items: HasMany<item::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
