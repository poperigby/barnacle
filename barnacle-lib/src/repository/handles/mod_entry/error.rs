//! Error types for mod entry handle operations.

use thiserror::Error;

use crate::repository::handles::error::{GetFieldError, LoadModelError, ModelKind};

impl super::ModEntry {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::ModEntry, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum RelatedModError {
    #[error("could not load mod entry")]
    LoadEntry(#[source] LoadModelError),
    #[error("could not load mod for entry")]
    LoadMod(#[source] sea_orm::DbErr),
    #[error("mod for entry no longer exists")]
    StaleMod,
}

#[derive(Debug, Error)]
pub enum RelatedProfileError {
    #[error("could not load mod entry")]
    LoadEntry(#[source] LoadModelError),
    #[error("could not load profile for entry")]
    LoadProfile(#[source] sea_orm::DbErr),
    #[error("profile for entry no longer exists")]
    StaleProfile,
}

#[derive(Debug, Error)]
#[error("could not get mod entry name")]
pub struct NameError(#[source] pub RelatedModError);

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
    #[error("could not get mod name")]
    ModName(#[source] RelatedModError),
    #[error("could not get profile name")]
    ProfileName(#[source] RelatedProfileError),
    #[error("could not delete mod entry")]
    Delete(#[source] sea_orm::DbErr),
}

#[derive(Debug, Error)]
#[error("could not query mod entries for listing")]
pub struct ListError(#[source] pub sea_orm::DbErr);
