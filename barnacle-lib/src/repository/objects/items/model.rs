use sea_orm::prelude::*;

use crate::repository::{items::kind::Kind, mods};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub mod_id: i32,
    #[sea_orm(belongs_to, from = "mod_id", to = "id", on_delete = "Cascade")]
    pub mod_: BelongsTo<mods::Entity>,

    pub kind: Kind,

    pub relative_path: String,
}

impl ActiveModelBehavior for ActiveModel {}
