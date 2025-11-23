use barnacle_lib::Repository;
use iced::{
    Color, Element, Length, Task,
    widget::{Column, center, container, mouse_area, opaque, stack},
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
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
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
            }))
            .on_press(on_blur)
        )
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
