use crate::icons::icon;
use barnacle_lib::state::State;
use iced::{
    Element, Task, Theme, application,
    widget::{button, column, horizontal_space, row, text},
};

mod icons;

fn main() -> iced::Result {
    application("Barnacle", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {}

struct App {
    state: State,
    theme: Theme,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let state = State::new().unwrap();

        (
            Self {
                state: state.clone(),
                theme: Theme::Dark,
            },
            Task::none(),
        )
    }

    // Update application state based on messages passed by view()
    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    // Render the application and pass along messages from components to update()
    fn view(&self) -> Element<'_, Message> {
        column![row![
            text("Game:"),
            button(icon("play")),
            text("Profile:"),
            horizontal_space(),
            button(icon("library")),
            button(icon("settings")),
            button(icon("notifications"))
        ]]
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
