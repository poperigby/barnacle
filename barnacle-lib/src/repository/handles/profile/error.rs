use std::{io, path::PathBuf};

use thiserror::Error;

use crate::repository::handles::{
    error::{GetFieldError, LoadModelError, ModelKind},
    game,
};

impl super::Profile {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Profile, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum SetNameError {
    #[error("could not load profile")]
    Load(#[source] LoadModelError),
    #[error("a profile named '{name}' already exists for this game")]
    Duplicate { name: String },
    #[error("could not update profile name")]
    Update(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
pub enum DirError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not resolve parent game directory")]
    ParentDir(#[source] game::DirError),
}

#[derive(Debug, Error)]
pub enum GeneratedDirError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not resolve parent game's generated directory")]
    ParentGeneratedDir(#[source] game::GeneratedDirError),
    #[error("could not great generated directory")]
    Create(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum ActivateError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not set active profile")]
    SetActiveProfile(#[source] game::SetActiveProfileIdError),
}

#[derive(Debug, Error)]
pub enum IsActiveError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not load active profile")]
    Active(#[source] ActiveError),
}

#[derive(Debug, Error)]
pub enum ActiveError {
    #[error("could not reconcile active state")]
    Resolve(#[source] ResolveActiveIdError),
}

#[derive(Debug, Error)]
pub enum ResolveActiveIdError {
    #[error("could not query active profile ID")]
    ActiveProfileId(#[source] LoadModelError),
    #[error("could not find fallback profile")]
    FindFallbackProfile(#[source] sea_orm::DbErr),
    #[error("could not set active profile ID")]
    SetActiveProfile(#[source] game::SetActiveProfileIdError),
}

#[derive(Debug, Error)]
pub enum ParentError {
    #[error("could not load profile")]
    Load(#[source] LoadModelError),
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("could not get profile name")]
    Name(#[source] GetFieldError),
    #[error("could not get profile directory")]
    Dir(#[source] DirError),
    #[error("could not delete profile")]
    Delete(#[source] sea_orm::DbErr),
    #[error("could not remove profile directory")]
    RemoveDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("a profile named '{name}' already exists for this game")]
    DuplicateName { name: String },
    #[error("could not insert profile")]
    Insert(#[source] sea_orm::DbErr),
    #[error("could not get profile directory")]
    Dir(#[source] DirError),
    #[error("could not create profile directory")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
#[error("could not query profiles for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);

#[derive(Debug, Error)]
#[error("could not find profile named '{name}'")]
pub struct SearchError {
    pub name: String,
    #[source]
    pub source: sea_orm::DbErr,
}
