use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadStateError {
    #[error("could not query repository state row")]
    Query(#[source] DbErr),
    #[error("could not create repository state row")]
    Create(#[source] DbErr),
}

#[derive(Debug, Error)]
pub enum SetActiveGameIdError {
    #[error("could not load repository state for updating active game")]
    LoadState(#[source] LoadStateError),
    #[error("could not update active game")]
    Update(#[source] DbErr),
}
