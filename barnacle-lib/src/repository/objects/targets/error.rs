//! Error types for target object operations.

use thiserror::Error;

use crate::repository::objects::error::{GetFieldError, LoadModelError, ModelKind};

impl super::Target {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Target, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("could not get target path")]
    Field(#[source] GetFieldError),
}

#[derive(Debug, Error)]
pub enum ParentError {
    #[error("could not load target")]
    Load(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("the supplied path is not valid UTF-8")]
    NotUtf8Path,
    #[error("could not retrieve the name of the given parent game")]
    GameName(#[source] GetFieldError),
    #[error("game '{game}' already contains a target named '{name}'")]
    DuplicateName { game: String, name: String },
    #[error("game '{game}' already contains a target with path of '{path}'")]
    DuplicatePath { game: String, path: String },
    #[error("could not insert target")]
    Insert(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not load parent game's name")]
    GameName(#[source] GetFieldError),
    #[error("could not load target name")]
    TargetName(#[source] GetFieldError),
    #[error("could not delete target")]
    Delete(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
#[error("could not query targets for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);
