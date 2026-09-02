use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique_key = "mod_name_per_game")]
    pub name: String,

    #[sea_orm(unique_key = "mod_name_per_game")]
    pub game_id: i32,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: BelongsTo<super::games::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
