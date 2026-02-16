use std::path::PathBuf;

use agdb::{DbElement, DbId};
use sea_orm::entity::prelude::*;

use crate::repository::entities::Uid;

#[derive(Debug, Clone, DbElement, PartialEq, PartialOrd)]
pub struct ToolModel {
    db_id: Option<DbId>,
    uid: u64,
    /// A human friendly display name
    name: String,
    /// The path to the tool's executable
    path: PathBuf,
    /// Additional command-line arguments
    args: Option<String>,
}

impl ToolModel {
    pub fn new(uid: Uid, name: &str, path: PathBuf, args: Option<&str>) -> Self {
        Self {
            db_id: None,
            uid: uid.0,
            name: name.to_string(),
            path,
            args: args.map(str::to_string),
        }
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tool")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: Option<i32>,
    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<super::games::Entity>,

    pub name: String,
    pub path: String,
    pub args: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
