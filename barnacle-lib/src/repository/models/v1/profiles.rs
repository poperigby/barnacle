use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub(crate) struct ProfileModel {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
}

impl ProfileModel {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
        }
    }
}
