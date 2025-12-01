use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, Default, PartialEq, PartialOrd)]
pub(crate) struct ModEntryModel {
    db_id: Option<DbId>,
    enabled: bool,
    notes: String,
}
