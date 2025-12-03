use barnacle_lib::Repository;
use iced::{
    Color, Element, Length, Task,
    widget::{center, container, mouse_area, opaque, stack},
};

pub mod config;
pub mod icons;

pub trait Component
where
    Self: Sized,
{
    type Message;

    fn new(repo: Repository) -> (Self, Task<Self::Message>);
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> Element<'_, Self::Message>;
}

/// Make an element modal, capturing mouse input and darkening the background.
pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_click_outside: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mouse_area = mouse_area(center(opaque(content)).style(|_theme| {
        container::Style {
            background: Some(
                Color {
                    a: 0.8,
                    ..Color::BLACK
                }
                .into(),
            ),
            ..container::Style::default()
        }
    }));

    stack![
        base.into(),
        opaque(if let Some(msg) = on_click_outside {
            mouse_area.on_press(msg)
        } else {
            mouse_area
        })
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// pub fn list<Item, Message>(items: &[Item], row_fn: Fn) -> Element<'_, Message> {
//     let children = items.iter().map(|i| row_fn(i));
//
//     Column::with_children(children).into()
// }
