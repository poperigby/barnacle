use agdb::{DbElement, DbId};
use sea_orm::entity::prelude::*;

use crate::repository::entities::Uid;

#[derive(Debug, Clone, DbElement, PartialEq, PartialOrd)]
pub(crate) struct ProfileModel {
    db_id: Option<DbId>,
    uid: u64,
    name: String,
}

impl ProfileModel {
    pub fn new(uid: Uid, name: &str) -> Self {
        Self {
            db_id: None,
            uid: uid.0,
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "profile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: Option<i32>,
    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<super::games::Entity>,

    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}
