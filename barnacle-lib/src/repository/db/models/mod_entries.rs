use agdb::{DbElement, DbId};
use sea_orm::entity::prelude::*;

use crate::repository::entities::Uid;

#[derive(Debug, Clone, DbElement, PartialEq, PartialOrd)]
pub(crate) struct ModEntryModel {
    db_id: Option<DbId>,
    uid: u64,
    enabled: bool,
    notes: String,
}

impl ModEntryModel {
    pub fn new(uid: Uid) -> Self {
        Self {
            db_id: None,
            uid: uid.0,
            enabled: true,
            notes: "".into(),
        }
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mod_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub mod_id: i32,
    #[sea_orm(primary_key)]
    pub profile_id: i32,

    #[sea_orm(unique)]
    pub position: i32,
    pub enabled: bool,
    pub notes: String,
}

impl ActiveModelBehavior for ActiveModel {}
