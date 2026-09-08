use std::{io, path::PathBuf};

use thiserror::Error;

use crate::repository::{
    handles::{
        error::{GetFieldError, LoadModelError, ModelKind},
        game,
    },
    state,
};

impl super::Profile {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Profile, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum SetNameError {
    #[error("could not get current profile directory")]
    CurrentDir(#[source] DirError),
    #[error("could not load profile")]
    Load(#[source] LoadModelError),
    #[error("a profile named '{name}' already exists for this game")]
    Duplicate { name: String },
    #[error("could not update profile name")]
    Update(#[source] sea_orm::DbErr),
    #[error("could not rename directory from '{from}' to '{to}'")]
    RenameDir {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum DirError {
    #[error("could not load parent game")]
    Parent(#[source] ParentError),
    #[error("could not resolve parent game directory")]
    ParentDir(#[source] game::DirError),
    #[error("could not get profile name")]
    Name(#[source] GetFieldError),
}

#[derive(Debug, Error)]
pub enum ActivateError {
    #[error("could not load active game ID")]
    ActiveGameId(#[source] state::error::LoadStateError),
    #[error("could not load profile")]
    Load(#[source] LoadModelError),
    #[error("profile does not belong to the active game")]
    ProfileNotInActiveGame,
    #[error("could not set active profile")]
    SetActiveProfile(#[source] state::error::SetActiveProfileIdError),
}

#[derive(Debug, Error)]
pub enum IsActiveError {
    #[error("could not load active profile ID")]
    ActiveProfileId(#[source] state::error::LoadStateError),
}

#[derive(Debug, Error)]
pub enum ActiveError {
    #[error("could not reconcile active state before loading active profile")]
    Reconcile(#[source] state::error::ReconcileError),
    #[error("could not load active profile ID")]
    ActiveProfileId(#[source] state::error::LoadStateError),
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
    #[error("could not reconcile active state after removing profile")]
    Reconcile(#[source] state::error::ReconcileError),
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
    #[error("could not reconcile active state after adding profile")]
    Reconcile(#[source] state::error::ReconcileError),
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
