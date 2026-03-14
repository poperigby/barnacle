use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mod_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub profile_id: i64,
    pub mod_id: i64,
    #[sea_orm(unique_key = "profile_position")]
    pub position: i64,
    pub enabled: bool,
    pub notes: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::profiles::Entity",
        from = "Column::ProfileId",
        to = "super::profiles::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Profile,
    #[sea_orm(
        belongs_to = "super::mods::Entity",
        from = "Column::ModId",
        to = "super::mods::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Mod,
}

impl Related<super::profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::mods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Mod.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
