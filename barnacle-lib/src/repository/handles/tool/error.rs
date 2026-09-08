//! Error types for tool handle operations.

use thiserror::Error;

use crate::repository::handles::error::{GetFieldError, LoadModelError, ModelKind};

impl super::Tool {
    pub(crate) fn field_error(&self, field: &'static str, source: LoadModelError) -> GetFieldError {
        GetFieldError::new(ModelKind::Tool, self.id, field, source)
    }
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("could not get tool path")]
    Field(#[source] GetFieldError),
}
