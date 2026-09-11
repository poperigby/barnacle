use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModList {
    pub sort_state: SortState,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum SortColumn {
    #[default]
    Name,
    Category,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}
