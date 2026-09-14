use sea_orm::prelude::*;

pub mod entries;
pub mod groups;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,

    /// Absolute path to the item, from the parent mod's root directory.
    pub path: String,

    pub group_id: i32,
    #[sea_orm(belongs_to, from = "group_id", to = "id", on_delete = "Cascade")]
    pub group: BelongsTo<super::items::groups::Entity>,

    pub mod_id: i32,
    #[sea_orm(belongs_to, from = "mod_id", to = "id", on_delete = "Cascade")]
    pub mod_: BelongsTo<super::mods::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
