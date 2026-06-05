use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    #[sea_orm(unique_key = "per_game_name")]
    pub game_id: i64,
    #[sea_orm(belongs_to, from = "game_id", to = "id", on_delete = "Cascade")]
    pub game: HasOne<super::games::Entity>,

    #[sea_orm(has_many)]
    pub mod_entry: HasMany<super::mod_entries::Entity>,

    #[sea_orm(unique_key = "per_game_name")]
    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}
