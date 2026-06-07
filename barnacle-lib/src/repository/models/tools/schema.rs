use sea_orm::entity::prelude::*;

use crate::models;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tools")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    // Fields
    pub name: String,
    pub path: String,
    pub args: Option<String>,

    // Parent
    pub game_id: i64,
    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<models::games::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
