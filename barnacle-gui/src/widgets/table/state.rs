use iced::{Point, Rectangle};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub columns: Vec<f32>,
    pub rows: Vec<f32>,
    /// Bounds the cover the entirety of row, for hit testing
    pub row_bounds: Vec<Rectangle>,
    pub selected_row: Option<usize>,
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
pub enum Over {
    Top(usize),
    Bottom(usize),
}

#[derive(Debug, Clone)]
pub struct Row {
    pub index: usize,
    pub bounds: Rectangle,
}

impl State {
    pub fn row_at(&self, relative_position: Point) -> Option<Row> {
        self.row_bounds
            .iter()
            .copied()
            .skip(1)
            .enumerate()
            .find(|(_, bounds)| bounds.contains(relative_position))
            .map(|(index, bounds)| Row { index, bounds })
    }
}
