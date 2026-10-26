use std::collections::BTreeSet;

use iced::{Point, Rectangle};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub columns: Vec<f32>,
    pub rows: Vec<f32>,
    /// Bounds the cover the entirety of row, for hit testing
    pub row_bounds: Vec<Rectangle>,
    pub selected_rows: BTreeSet<usize>,
    pub dragging: DraggingState,
}

#[derive(Debug, Clone, Default)]
pub enum DraggingState {
    #[default]
    /// No iteraction
    Idle,
    /// Selection button is held over row, but hasn't moved
    Held { row_index: usize },
    /// Selection button is held and cursor is moving
    Dragging {
        row_index: usize,
        position: Point,
        over: Option<Over>,
    },
}

#[derive(Debug, Clone)]
pub struct Row {
    pub index: usize,
    pub bounds: Rectangle,
}

impl State {
    pub fn row_at(&self, position: Point) -> Option<Row> {
        self.row_bounds
            .iter()
            .copied()
            // Skip the header row
            .skip(1)
            .enumerate()
            .find(|(_, bounds)| bounds.contains(position))
            .map(|(index, bounds)| Row { index, bounds })
    }

    /// Determines whether a given [`Point`] is over the top or bottom of a row.
    pub fn over_at(&self, position: Point) -> Option<Over> {
        self.row_at(position).map(|hovered_row| {
            let midpoint = hovered_row.bounds.y + hovered_row.bounds.height / 2.0;

            if position.y < midpoint {
                Over::Top(hovered_row.index)
            } else {
                Over::Bottom(hovered_row.index)
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Over {
    Top(usize),
    Bottom(usize),
}

impl Over {
    // Converts the hovered target into a final position.
    // Returns [`None`] if the row would stay in place.
    pub fn destination(&self, dragged_row_index: usize) -> Option<usize> {
        // Determine the index of the gap between each row
        // e.g. gap 0 | A | gap 1 | B | gap 2 | C | gap 3 | D | gap 4
        // Top(B) and Bottom(A) both choose gap 1
        let gap_index = match self {
            Over::Top(index) => *index,
            Over::Bottom(index) => *index + 1,
        };

        let new_position = if dragged_row_index < gap_index {
            gap_index - 1
        } else {
            gap_index
        };

        // Ignore moves that would leave the row in place
        if new_position != dragged_row_index {
            Some(new_position)
        } else {
            None
        }
    }
}
