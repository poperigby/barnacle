use sea_orm::prelude::*;

use crate::repository::mods;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,

    pub group: String,

    /// Relative path to the item, from the parent mod's root directory.
    pub relative_path: String,

    pub mod_id: i32,
    #[sea_orm(belongs_to, from = "mod_id", to = "id", on_delete = "Cascade")]
    pub mod_: BelongsTo<mods::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
