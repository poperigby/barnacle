use sea_orm::prelude::*;

use crate::repository::item_groups;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique_key = "key_per_group")]
    pub group_id: i32,
    #[sea_orm(belongs_to, from = "group_id", to = "id", on_delete = "Cascade")]
    pub group: BelongsTo<item_groups::Entity>,

    /// Stable identifier for an [`Item`]. For example, an `.esp` file name or a module identifier.
    #[sea_orm(unique_key = "key_per_group")]
    pub key: String,
}

impl ActiveModelBehavior for ActiveModel {}
