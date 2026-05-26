use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    // Composite unique key: mod names only need to be unique within a game.
    #[sea_orm(unique_key = "per_game_name")]
    pub game_id: i64,
    #[sea_orm(unique_key = "per_game_name")]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::games::Entity",
        from = "Column::GameId",
        to = "super::games::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Game,
    #[sea_orm(has_many = "super::mod_entries::Entity")]
    ModEntries,
}

impl Related<super::games::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
