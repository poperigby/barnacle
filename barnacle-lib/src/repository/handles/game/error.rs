use std::{io, path::PathBuf};

use thiserror::Error;

use crate::repository::{
    handles::error::{GetFieldError, LoadModelError, ModelKind},
    state,
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
    #[error("could not set active game ID")]
    SetActiveGameId(#[source] state::error::SetActiveGameIdError),
}

#[derive(Debug, Error)]
pub enum IsActiveError {
    #[error("could not retrieve active game")]
    Active(#[source] ActiveError),
}

#[derive(Debug, Error)]
pub enum ActiveError {
    #[error("could not resolve active game ID")]
    Resolve(#[source] ResolveActiveIdError),
}

#[derive(Debug, Error)]
pub enum ResolveActiveIdError {
    #[error("could not load active game ID")]
    ActiveGameId(#[source] state::error::LoadStateError),
    #[error("could not find fallback game")]
    FindFallbackGame(#[source] sea_orm::DbErr),
    #[error("could not set active game ID")]
    SetActiveGameId(#[source] state::error::SetActiveGameIdError),
}

#[derive(Debug, Error)]
pub enum SetActiveProfileIdError {
    #[error("could not load game")]
    Load(#[source] LoadModelError),
    #[error("could not update active profile ID")]
    Update(#[source] sea_orm::DbErr),
}
