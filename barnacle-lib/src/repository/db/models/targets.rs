use sea_orm::prelude::*;

use crate::repository::db::models::mods;

/// Represents a destination where mod files are deployed to.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "targets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique_key = "target_name_per_game")]
    pub name: String,
    #[sea_orm(unique_key = "target_path_per_game")]
    pub path: String,

    #[sea_orm(unique_key = "target_name_per_game")]
    #[sea_orm(unique_key = "target_path_per_game")]
    pub game_id: i32,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: BelongsTo<super::games::Entity>,

    #[sea_orm(has_many)]
    pub mod_entries: HasMany<mods::entries::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
