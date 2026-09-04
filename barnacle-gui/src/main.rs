use fluent_i18n::i18n;
use iced::{
    Color, Element,
    Length::{self},
    application,
    widget::{center, container, mouse_area, opaque, stack},
    window::Settings,
};
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app::App;

pub mod app;
pub mod config;
pub mod icons;

i18n!("locales", fallback = "en-US");

fn main() -> iced::Result {
    human_panic::setup_panic!();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut settings = Settings::default();
    settings.platform_specific.application_id = App::TITLE.to_string();

    application(App::new, App::update, App::view)
        .theme(App::theme)
        .title(App::title)
        .window(settings)
        .run()
}

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
