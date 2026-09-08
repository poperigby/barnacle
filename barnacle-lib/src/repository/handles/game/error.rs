use std::{io, path::PathBuf};

use thiserror::Error;

use crate::repository::{
    handles::{
        error::{GetFieldError, LoadModelError, ModelKind},
        mod_ as mod_handle, profile,
    },
    state::{self, error::ReconcileError},
};

impl super::Game {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Game, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum SetNameError {
    #[error("could not get current game directory")]
    CurrentDir(#[source] DirError),
    #[error("could not load game")]
    Load(#[source] LoadModelError),
    #[error("a game named '{name}' already exists")]
    Duplicate { name: String },
    #[error("could not update game name")]
    Update(#[source] sea_orm::DbErr),
    #[error("could not complete game rename transaction")]
    Transaction(#[source] sea_orm::DbErr),
    #[error("could not rename directory from '{from}' to {to}")]
    RenameDir {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum SetDeployKindError {
    #[error("could not load game")]
    Load(#[source] LoadModelError),
    #[error("could not update game deploy kind")]
    Update(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
#[error("could not resolve game directory")]
pub struct DirError {
    #[source]
    pub source: GetFieldError,
}

#[derive(Debug, Error)]
pub enum RemoveError {
    #[error("could not get game name")]
    Name(#[source] GetFieldError),
    #[error("could not get game directory")]
    Dir(#[source] DirError),
    #[error("could not delete game")]
    Delete(#[source] sea_orm::DbErr),
    #[error("could not reconcile active state after removing game")]
    Reconcile(#[source] ReconcileError),
    #[error("could not complete game removal transaction")]
    Transaction(#[source] sea_orm::DbErr),
    #[error("could not remove game directory")]
    RemoveDir {
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("a game named '{name}' already exists")]
    DuplicateName { name: String },
    #[error("could not insert game")]
    Insert(#[source] sea_orm::DbErr),
    #[error("could not reconcile active state after adding game")]
    Reconcile(#[source] ReconcileError),
    #[error("could not complete game creation transaction")]
    Transaction(#[source] sea_orm::DbErr),
    #[error("could not create directory")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
#[error("could not query games for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);

#[derive(Debug, Error)]
#[error("could not find game named '{name}'")]
pub struct SearchError {
    pub name: String,

    #[source]
    pub source: sea_orm::DbErr,
}

#[derive(Debug, Error)]
pub enum ActivateError {
    #[error("could not set active game")]
    SetActiveGame(#[source] state::error::SetActiveGameIdError),
}

#[derive(Debug, Error)]
pub enum IsActiveError {
    #[error("could not load active game ID")]
    ActiveGameId(#[source] state::error::LoadStateError),
}

#[derive(Debug, Error)]
pub enum ActiveError {
    #[error("could not reconcile active state before loading active game")]
    Reconcile(#[source] state::error::ReconcileError),

    #[error("could not load active game ID")]
    ActiveGameId(#[source] state::error::LoadStateError),
}

#[derive(Debug, Error)]
#[error("could not load active profile for game")]
pub struct ActiveProfileError(#[source] pub profile::ActiveError);

#[derive(Debug, Error)]
#[error("could not search profiles for game")]
pub struct SearchProfileError(#[source] pub profile::SearchError);

#[derive(Debug, Error)]
#[error("could not add profile to game")]
pub struct AddProfileError(#[source] pub profile::AddError);

#[derive(Debug, Error)]
#[error("could not list profiles for game")]
pub struct ProfilesError(#[source] pub profile::ListError);

#[derive(Debug, Error)]
#[error("could not list mods for game")]
pub struct ModsError(#[source] pub mod_handle::ListError);

#[derive(Debug, Error)]
#[error("could not add mod to game")]
pub struct AddModError(#[source] pub mod_handle::AddError);
