use sea_orm::prelude::*;

use crate::repository::{item_entries, profiles};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "item_lists")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,

    #[sea_orm(unique_key = "group_per_profile")]
    pub group: String,

    pub sortable: bool,

    #[sea_orm(unique_key = "group_per_profile")]
    pub profile_id: i32,
    #[sea_orm(belongs_to, from = "profile_id", to = "id", on_delete = "Cascade")]
    pub profile: BelongsTo<profiles::Entity>,

    #[sea_orm(has_many)]
    pub item_entries: HasMany<item_entries::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
