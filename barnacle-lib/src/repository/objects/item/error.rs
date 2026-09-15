use thiserror::Error;

use crate::error::{GetFieldError, LoadModelError, ModelKind};

impl super::ItemEntry {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::ItemEntry, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum ParentError {
    #[error("could not load item entry")]
    Load(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("could not find next item entry priority")]
    NextPriority(#[source] sea_orm::DbErr),
    #[error("profile already contains this item")]
    Duplicate,
    #[error("could not insert item entry")]
    Insert(#[source] sea_orm::DbErr),
    #[error("could not get mod name for logging")]
    ModName(#[source] GetFieldError),
    #[error("could not get profile name for logging")]
    ProfileName(#[source] GetFieldError),
}
