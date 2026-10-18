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
    Held { row: usize, origin: Point },
    /// Selection button is held and cursor is moving
    Dragging {
        row: usize,
        origin: Point,
        current: Point,
    },
}
