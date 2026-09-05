use iced::Element;
use serde::{Deserialize, Serialize};

use crate::{icons::Icon, workspace::mod_list::Message};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Category,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl SortState {
    pub fn toggle(&self, column: SortColumn) -> Self {
        if self.column == column {
            let new_direction = match self.direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };

            Self {
                column,
                direction: new_direction,
            }
        } else {
            // A different column than the currently sorted one has been selected
            Self {
                column,
                ..Default::default()
            }
        }
    }

    pub fn icon(&'_ self, column: SortColumn) -> Option<Element<'_, Message>> {
        if self.column == column {
            Some(
                match self.direction {
                    SortDirection::Ascending => Icon::ArrowUp,
                    SortDirection::Descending => Icon::ArrowDown,
                }
                .into(),
            )
        } else {
            None
        }
    }
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Name,
            direction: SortDirection::Ascending,
        }
    }
}
