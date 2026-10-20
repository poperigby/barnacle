//! Display tables.
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay,
    renderer::{self, Quad},
    widget,
};
use iced::{
    Alignment, Background, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector,
    alignment, mouse,
};

use crate::widgets::table::state::{DraggingState, Over, State};

mod state;

/// Creates a new [`Table`] with the given columns and rows.
///
/// Columns can be created using the [`column()`] function, while rows can be any
/// iterator over some data type `T`.
pub fn table<'a, 'b, T, Message, Theme, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> Table<'a, Message, Theme, Renderer>
where
    T: Clone,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    Table::new(columns, rows)
}

/// Creates a new [`Column`] with the given header and view function.
///
/// The view function will be called for each row in a [`Table`] and it must
/// produce the resulting contents of a cell.
pub fn column<'a, 'b, T, E, Message, Theme, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(T) -> E + 'b,
) -> Column<'a, 'b, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header: header.into(),
        view: Box::new(move |data| view(data).into()),
        width: Length::Shrink,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Top,
    }
}

#[derive(Debug, Clone)]
pub struct RowMoved {
    pub index: usize,
    pub new_position: usize,
}

/// A grid-like visual representation of data distributed in columns and rows.
pub struct Table<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    columns: Vec<Column_>,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    class: Theme::Class<'a>,
    on_right_click_row: Option<Box<dyn Fn(usize, Point) -> Message + 'a>>,
    on_row_moved: Option<Box<dyn Fn(RowMoved) -> Message + 'a>>,
}

struct Column_ {
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new [`Table`] with the given columns and rows.
    ///
    /// Columns can be created using the [`column()`] function, while rows can be any
    /// iterator over some data type `T`.
    pub fn new<'b, T>(
        columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
        rows: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Clone,
    {
        let columns = columns.into_iter();
        let rows = rows.into_iter();

        let mut width = Length::Shrink;
        let mut height = Length::Shrink;

        let mut cells = Vec::with_capacity(columns.size_hint().0 * (1 + rows.size_hint().0));

        let (mut columns, views): (Vec<_>, Vec<_>) = columns
            .map(|column| {
                width = width.enclose(column.width);

                cells.push(column.header);

                (
                    Column_ {
                        width: column.width,
                        align_x: column.align_x,
                        align_y: column.align_y,
                    },
                    column.view,
                )
            })
            .collect();

        for row in rows {
            for view in &views {
                let cell = view(row.clone());
                let size_hint = cell.as_widget().size_hint();

                height = height.enclose(size_hint.height);

                cells.push(cell);
            }
        }

        if width == Length::Shrink
            && let Some(first) = columns.first_mut()
        {
            first.width = Length::Fill;
        }

        Self {
            columns,
            cells,
            width,
            height,
            padding_x: 10.0,
            padding_y: 5.0,
            separator_x: 1.0,
            separator_y: 1.0,
            class: Theme::default(),
            on_right_click_row: None,
            on_row_moved: None,
        }
    }

    pub fn on_right_click_row<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(usize, Point) -> Message,
    {
        self.on_right_click_row = Some(Box::new(f));
        self
    }

    pub fn on_row_moved<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(RowMoved) -> Message,
    {
        self.on_row_moved = Some(Box::new(f));
        self
    }

    /// Sets the width of the [`Table`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding of the cells of the [`Table`].
    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    /// Sets the horizontal padding of the cells of the [`Table`].
    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    /// Sets the vertical padding of the cells of the [`Table`].
    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    /// Sets the thickness of the line separator between the cells of the [`Table`].
    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    /// Sets the thickness of the horizontal line separator between the cells of the [`Table`].
    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    /// Sets the thickness of the vertical line separator between the cells of the [`Table`].
    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.cells
            .iter()
            .map(|cell| widget::Tree::new(cell.as_widget()))
            .collect()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&self.cells);
    }

    #[expect(
        clippy::integer_division,
        reason = "Cells form complete rows, integer division computes row counts and indices."
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "Cell indices determine rows and columns, metric arrays are sized to that same grid."
    )]
    #[expect(
        clippy::cast_precision_loss,
        reason = "Grid dimensions are converted to approximate f32 pixel coordinates for layout."
    )]
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = tree.state.downcast_mut::<State>();
        let columns = self.columns.len();
        let limits = limits.width(self.width).height(self.height);

        if columns == 0 {
            metrics.columns.clear();
            metrics.rows.clear();
            return layout::Node::new(limits.resolve(self.width, self.height, Size::ZERO));
        }

        let rows = self.cells.len() / columns;

        let available = limits.max();
        let table_fluid = self.width.fluid();

        let mut cells = Vec::with_capacity(self.cells.len());
        cells.resize(self.cells.len(), layout::Node::default());

        metrics.columns = vec![0.0; self.columns.len()];
        metrics.rows = vec![0.0; rows];

        let mut column_factors = vec![0; self.columns.len()];
        let mut total_row_factors = 0;
        let mut total_fluid_height = 0.0;
        let mut row_factor = 0;

        let spacing_x = self.padding_x * 2.0 + self.separator_x;
        let spacing_y = self.padding_y * 2.0 + self.separator_y;

        // FIRST PASS
        // Lay out non-fluid cells
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            let width = self.columns[column].width;
            let size = cell.as_widget().size();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;

                    if row_factor != 0 {
                        total_fluid_height += metrics.rows[row - 1];
                        total_row_factors += row_factor;

                        row_factor = 0;
                    }
                }
            }

            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if width_factor != 0 || height_factor != 0 || size.width.is_fill() {
                column_factors[column] = column_factors[column].max(width_factor);

                row_factor = row_factor.max(height_factor);

                continue;
            }

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(available.width - x, available.height - y),
            )
            .width(width);

            let layout = cell.as_widget_mut().layout(state, renderer, &limits);
            let size = limits.resolve(width, Length::Shrink, layout.size());

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // SECOND PASS
        // Lay out fluid cells, using metrics from the first pass as limits
        let left = Size::new(
            available.width
                - metrics
                    .columns
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| column_factors[*i] == 0)
                    .map(|(_, width)| width)
                    .sum::<f32>(),
            available.height - total_fluid_height,
        );

        let width_unit = (left.width
            - spacing_x * self.columns.len().saturating_sub(1) as f32
            - self.padding_x * 2.0)
            / column_factors.iter().sum::<u16>() as f32;

        let height_unit =
            (left.height - spacing_y * rows.saturating_sub(1) as f32 - self.padding_y * 2.0)
                / total_row_factors as f32;

        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in self.cells.iter_mut().zip(&mut tree.children).enumerate() {
            let row = i / columns;
            let column = i % columns;

            let size = cell.as_widget().size();

            let width = self.columns[column].width;
            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            if width_factor == 0 && size.width.fill_factor() == 0 && size.height.fill_factor() == 0
            {
                continue;
            }

            let max_width = if width_factor == 0 {
                if size.width.is_fill() {
                    metrics.columns[column]
                } else {
                    (available.width - x).max(0.0)
                }
            } else {
                width_unit * width_factor as f32
            };

            let max_height = if height_factor == 0 {
                if size.height.is_fill() {
                    metrics.rows[row]
                } else {
                    (available.height - y).max(0.0)
                }
            } else {
                height_unit * height_factor as f32
            };

            let limits =
                layout::Limits::new(Size::ZERO, Size::new(max_width, max_height)).width(width);

            let layout = cell.as_widget_mut().layout(state, renderer, &limits);
            let size = limits.resolve(
                if let Length::Fixed(_) = width {
                    width
                } else {
                    table_fluid
                },
                Length::Shrink,
                layout.size(),
            );

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // THIRD PASS
        // Position each cell
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, cell) in cells.iter_mut().enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            let Column_ {
                align_x, align_y, ..
            } = &self.columns[column];

            cell.move_to_mut((x, y));
            cell.align_mut(
                Alignment::from(*align_x),
                Alignment::from(*align_y),
                Size::new(metrics.columns[column], metrics.rows[row]),
            );

            x += metrics.columns[column] + spacing_x;
        }

        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                x - spacing_x + self.padding_x,
                y + metrics
                    .rows
                    .last()
                    .copied()
                    .map(|height| height + self.padding_y)
                    .unwrap_or_default(),
            ),
        );

        let mut y = 0.0;
        metrics.row_bounds = metrics
            .rows
            .iter()
            .map(|height| {
                let bounds = Rectangle {
                    x: 0.0,
                    y,
                    width: intrinsic.width,
                    height: height + self.padding_y * 2.0,
                };

                y += bounds.height + self.separator_y;
                bounds
            })
            .collect();

        layout::Node::with_children(intrinsic, cells)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Some(cursor_local_pos) = cursor.position_in(layout.bounds()) {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    button @ (mouse::Button::Left | mouse::Button::Right),
                )) => {
                    let row_index = state.row_at(cursor_local_pos).map(|r| r.index);

                    match button {
                        mouse::Button::Left => {
                            state.dragging = match row_index {
                                Some(row) => DraggingState::Held { row_index: row },
                                None => DraggingState::Idle,
                            };

                            state.selected_row = row_index;
                            shell.request_redraw();
                        }
                        mouse::Button::Right => {
                            if let Some(row) = row_index
                                && let Some(callback) = &self.on_right_click_row
                            {
                                shell.publish(callback(row, cursor_local_pos));
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => match state.dragging {
                    DraggingState::Held { row_index: row }
                    | DraggingState::Dragging { row_index: row, .. } => {
                        let over = state.row_at(cursor_local_pos).map(|target| {
                            let midpoint = target.bounds.y + target.bounds.height / 2.0;

                            if cursor_local_pos.y < midpoint {
                                Over::Top(target.index)
                            } else {
                                Over::Bottom(target.index)
                            }
                        });

                        state.dragging = DraggingState::Dragging {
                            row_index: row,
                            position: *position,
                            over,
                        };
                        shell.request_redraw();
                    }
                    DraggingState::Idle => {}
                },
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    // While dragging, if cursor is over top half of another row, insert above. If bottom half,
                    // below.
                    if let DraggingState::Dragging {
                        row_index, over, ..
                    } = &state.dragging
                        && let Some(over) = over
                        && let Some(callback) = &self.on_row_moved
                    {
                        match over {
                            Over::Top(hovered_index) => {
                                shell.publish(callback(RowMoved {
                                    index: *row_index,
                                    new_position: *hovered_index,
                                }));
                            }
                            Over::Bottom(hovered_index) => {
                                shell.publish(callback(RowMoved {
                                    index: *row_index,
                                    new_position: *hovered_index + 1,
                                }));
                            }
                        };
                    }

                    state.dragging = DraggingState::Idle;

                    shell.request_redraw();
                }
                _ => {}
            }
        }

        for ((cell, tree), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let table_style = theme.style(&self.class);

        if let Some(selected_row) = state.selected_row
            // Exclude the header
            && let Some(row_bounds) = state.row_bounds.get(selected_row + 1)
        {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x + row_bounds.x,
                        y: bounds.y + row_bounds.y,
                        ..*row_bounds
                    },
                    ..Quad::default()
                },
                table_style.selected_row,
            );
        }

        for ((cell, state), layout) in self.cells.iter().zip(&tree.children).zip(layout.children())
        {
            cell.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        if self.separator_x > 0.0 {
            let mut x = self.padding_x;

            for width in state
                .columns
                .iter()
                .take(state.columns.len().saturating_sub(1))
            {
                x += width + self.padding_x;

                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: bounds.x + x,
                            y: bounds.y,
                            width: self.separator_x,
                            height: bounds.height,
                        },
                        snap: true,
                        ..Quad::default()
                    },
                    table_style.separator_x,
                );

                x += self.separator_x + self.padding_x;
            }
        }

        if self.separator_y > 0.0 {
            let mut y = self.padding_y;

            for (row_index, height) in state
                .rows
                .iter()
                .enumerate()
                .take(state.rows.len().saturating_sub(1))
            {
                y += height + self.padding_y;

                let background = match &state.dragging {
                    DraggingState::Dragging {
                        over: Some(Over::Top(index)),
                        ..
                    } if row_index == *index => table_style.seperator_hovering,
                    DraggingState::Dragging {
                        over: Some(Over::Bottom(index)),
                        ..
                    } if row_index == *index + 1 => table_style.seperator_hovering,
                    _ => table_style.separator_y,
                };

                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y + y,
                            width: bounds.width,
                            height: self.separator_y,
                        },
                        snap: true,
                        ..Quad::default()
                    },
                    background,
                );

                y += self.separator_y + self.padding_y;
            }
        }

        if let DraggingState::Dragging {
            position: current, ..
        } = state.dragging
        {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: current.x,
                        y: current.y,
                        width: layout.bounds().width * 0.75,
                        height: 30.0,
                    },
                    ..Quad::default()
                },
                table_style.selected_row,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((cell, tree), layout)| {
                cell.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        for ((cell, state), layout) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            cell.as_widget_mut()
                .operate(state, layout, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.cells,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(table)
    }
}

/// A vertical visualization of some data with a header.
pub struct Column<'a, 'b, T, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    header: Element<'a, Message, Theme, Renderer>,
    view: Box<dyn Fn(T) -> Element<'a, Message, Theme, Renderer> + 'b>,
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, 'b, T, Message, Theme, Renderer> Column<'a, 'b, T, Message, Theme, Renderer> {
    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the alignment for the horizontal axis of the [`Column`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the alignment for the vertical axis of the [`Column`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }
}

/// The appearance of a [`Table`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background color of the vertical line separator between cells.
    pub separator_x: Background,
    /// The background color of the horizontal line separator between cells.
    pub separator_y: Background,
    /// The background color of the horizontal separator when hovering near it.
    pub seperator_hovering: Background,
    /// The background color of the selected row.
    pub selected_row: Background,
}

/// The theme catalog of a [`Table`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Table`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default style of a [`Table`].
pub fn default(theme: &iced::Theme) -> Style {
    let palette = theme.extended_palette();
    let separator = palette.background.strong.color.into();

    Style {
        separator_x: separator,
        separator_y: separator,
        selected_row: palette.primary.weak.color.into(),
        seperator_hovering: palette.success.strong.color.into(),
    }
}
