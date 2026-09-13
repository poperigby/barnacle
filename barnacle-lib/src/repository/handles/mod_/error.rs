//! Error types for mod handle operations.

use std::{io, path::PathBuf};

use thiserror::Error;

use crate::repository::handles::{
    error::{GetFieldError, LoadModelError, ModelKind},
    game,
};

impl super::Mod {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Mod, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum DirError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not resolve parent game directory")]
    ParentDir(#[source] game::DirError),
}

#[derive(Debug, Error)]
pub enum ParentError {
    #[error("could not load mod")]
    Load(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("a mod named '{name}' already exists for this game")]
    DuplicateName { name: String },
    #[error("could not insert mod")]
    Insert(#[source] sea_orm::DbErr),
    #[error("could not open archive at '{path}'")]
    OpenArchive {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not get mod directory")]
    Dir(#[source] DirError),
    #[error("could not extract archive")]
    ExtractArchive(#[source] compress_tools::Error),
    #[error("could not create mod directory")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
#[error("could not query mods for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("could not get mod name")]
    Name(#[source] GetFieldError),
    #[error("could not get mod directory")]
    Dir(#[source] DirError),
    #[error("could not delete mod")]
    Delete(#[source] sea_orm::DbErr),
    #[error("could not remove mod directory")]
    RemoveDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
