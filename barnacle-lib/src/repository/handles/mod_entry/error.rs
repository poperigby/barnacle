//! Error types for mod entry handle operations.

use thiserror::Error;

use crate::{
    mod_,
    repository::handles::error::{GetFieldError, LoadModelError, ModelKind},
};

impl super::ModEntry {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::ModEntry, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("could not load mod entry to query mod_id")]
    LoadEntry(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum NameError {
    #[error("could not load related mod")]
    Load(#[source] ModError),
    #[error("could not query name field from mod")]
    Name(#[source] GetFieldError),
}

#[derive(Debug, Error)]
pub enum DirError {
    #[error("could not load related mod")]
    Load(#[source] ModError),
    #[error("could not query directory field from mod")]
    Dir(#[source] mod_::DirError),
}

#[derive(Debug, Error)]
pub enum SetEnabledError {
    #[error("could not load mod entry")]
    Load(#[source] LoadModelError),
    #[error("could not update mod entry enabled state")]
    Update(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
pub enum ParentError {
    #[error("could not load mod entry")]
    Load(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("could not find next mod entry priority")]
    NextPriority(#[source] sea_orm::DbErr),
    #[error("profile already contains this mod")]
    Duplicate,
    #[error("could not insert mod entry")]
    Insert(#[source] sea_orm::DbErr),
    #[error("could not get mod name for logging")]
    ModName(#[source] GetFieldError),
    #[error("could not get profile name for logging")]
    ProfileName(#[source] GetFieldError),
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("could not load related mod")]
    Mod(#[source] ModError),
    #[error("could not load related mod's name")]
    ModName(#[source] GetFieldError),
    #[error("could not load parent profile")]
    Profile(#[source] ParentError),
    #[error("could not get parent profile's name")]
    ProfileName(#[source] GetFieldError),
    #[error("could not delete mod entry")]
    Delete(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
#[error("could not query mod entries for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);
