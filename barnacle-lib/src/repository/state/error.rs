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

    #[error("could not reconcile active state after updating active game")]
    Reconcile(#[source] ReconcileError),
}

#[derive(Debug, Error)]
pub enum SetActiveProfileIdError {
    #[error("could not load repository state for updating active profile")]
    LoadState(#[source] LoadStateError),

    #[error("could not update active profile")]
    Update(#[source] DbErr),

    #[error("could not reconcile active state after updating active profile")]
    Reconcile(#[source] ReconcileError),
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("could not load active game id")]
    ActiveGameId(#[source] LoadStateError),

    #[error("could not load active profile id")]
    ActiveProfileId(#[source] LoadStateError),

    #[error("could not load active game {id}")]
    LoadActiveGame {
        id: i32,
        #[source]
        source: DbErr,
    },

    #[error("could not load active profile {id}")]
    LoadActiveProfile {
        id: i32,
        #[source]
        source: DbErr,
    },

    #[error("could not select fallback game")]
    SelectFallbackGame(#[source] SelectFallbackGameError),

    #[error("could not select fallback profile for game {game_id}")]
    SelectFallbackProfile {
        game_id: i32,
        #[source]
        source: SelectFallbackProfileError,
    },
}

#[derive(Debug, Error)]
pub enum SelectFallbackGameError {
    #[error("could not find fallback game")]
    FindFallbackGame(#[source] DbErr),
    #[error("could not load repository state for selecting fallback game")]
    LoadState(#[source] LoadStateError),
    #[error("could not set active game")]
    Update(#[source] DbErr),
}

#[derive(Debug, Error)]
pub enum SelectFallbackProfileError {
    #[error("could not find fallback profile for game {game_id}")]
    FindFallbackProfile {
        game_id: i32,
        #[source]
        source: DbErr,
    },
    #[error("could not load repository state for selecting fallback profile")]
    LoadState(#[source] LoadStateError),
    #[error("could not set active profile for game {game_id}")]
    Update {
        game_id: i32,
        #[source]
        source: DbErr,
    },
}
