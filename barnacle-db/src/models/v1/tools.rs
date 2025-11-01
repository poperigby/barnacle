use std::path::PathBuf;

use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Tool {
    db_id: Option<DbId>,
    /// A human friendly display name
    name: String,
    /// The path to the tool's executable
    path: PathBuf,
    /// Additional command-line arguments
    args: Option<String>,
}

impl Tool {
    pub fn new(name: &str, path: PathBuf, args: Option<&str>) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
            path,
            args: args.map(str::to_string),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
