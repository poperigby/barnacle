use sea_orm::SqlErr;
use strum::Display;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Display)]
pub enum ModelKind {
    Game,
    Profile,
    Mod,
    ModEntry,
    Tool,
    Target,
}

#[derive(Debug, Error)]
pub enum LoadModelError {
    #[error("{kind} {id} no longer exists")]
    Stale { kind: ModelKind, id: i32 },

    #[error("could not load {kind} {id}")]
    Query {
        kind: ModelKind,
        id: i32,
        #[source]
        source: sea_orm::DbErr,
    },
}

impl LoadModelError {
    pub(crate) fn stale(kind: ModelKind, id: i32) -> Self {
        Self::Stale { kind, id }
    }

    pub(crate) fn query(kind: ModelKind, id: i32, source: sea_orm::DbErr) -> Self {
        Self::Query { kind, id, source }
    }
}

#[derive(Debug, Error)]
#[error("could not get '{field}' for {kind} {id}")]
pub struct GetFieldError {
    kind: ModelKind,
    id: i32,
    field: &'static str,
    #[source]
    source: LoadModelError,
}

impl GetFieldError {
    pub(crate) fn new(
        kind: ModelKind,
        id: i32,
        field: &'static str,
        source: LoadModelError,
    ) -> Self {
        Self {
            kind,
            id,
            field,
            source,
        }
    }
}

pub(crate) fn is_unique_violation(err: &sea_orm::DbErr) -> bool {
    matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

pub(crate) fn map_transaction_error<E>(
    source: sea_orm::TransactionError<E>,
    transaction_error: fn(sea_orm::DbErr) -> E,
) -> E {
    match source {
        // The closure returned an error
        sea_orm::TransactionError::Transaction(source) => source,
        // Failed to connect to the database. Run a user provided closure to wrap the
        // underlying DbErr error in an error variant, such as FooError::Transaction(DbErr).
        sea_orm::TransactionError::Connection(source) => transaction_error(source),
    }
}
