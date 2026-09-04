//! Singleton reponsible for storing application state.

// TODO: Ensure the active profile belongs to the active game through the database

use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,

    pub active_game_id: Option<i32>,
    #[sea_orm(belongs_to, from = "active_game_id", to = "id", on_delete = "SetNull")]
    pub active_game: BelongsTo<Option<super::games::Entity>>,

    pub active_profile_id: Option<i32>,
    #[sea_orm(
        belongs_to,
        from = "active_profile_id",
        to = "id",
        on_delete = "SetNull"
    )]
    pub active_profile: BelongsTo<Option<super::profiles::Entity>>,
}

impl ActiveModelBehavior for ActiveModel {}
