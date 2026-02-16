use agdb::{DbElement, DbId};
use sea_orm::entity::prelude::*;

use crate::repository::entities::Uid;

#[derive(Debug, Clone, DbElement, PartialEq, PartialOrd)]
pub(crate) struct ModModel {
    db_id: Option<DbId>,
    uid: u64,
    /// A human friendly display name
    name: String,
}

impl ModModel {
    pub fn new(uid: Uid, name: &str) -> Self {
        Self {
            db_id: None,
            uid: uid.0,
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mod")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: Option<i32>,
    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<super::games::Entity>,

    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}
