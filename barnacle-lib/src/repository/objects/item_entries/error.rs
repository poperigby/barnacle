use thiserror::Error;

use crate::repository::items::Kind;

#[derive(Debug, Clone, Error)]
pub enum AutoSortError {
    #[error("the given item kind {0} does not support automatic sorting")]
    NotSupported(Kind),
}
