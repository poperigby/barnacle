use sea_orm::prelude::*;

use crate::repository::{games, mod_entries};

/// Represents a destination where mod files are deployed to.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "targets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,
    #[sea_orm(unique_key = "path_per_game")]
    pub path: String,

    #[sea_orm(unique_key = "path_per_game")]
    pub game_id: i32,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: BelongsTo<games::Entity>,

    #[sea_orm(has_many)]
    pub mod_entries: HasMany<mod_entries::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
