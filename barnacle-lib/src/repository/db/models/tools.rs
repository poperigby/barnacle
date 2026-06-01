use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tools")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub game_id: i64,
    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<super::games::Entity>,

    pub name: String,
    pub path: String,
    pub args: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
