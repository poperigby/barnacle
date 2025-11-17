use crate::{icons::icon, mod_list::ModList};
use barnacle_lib::Repository;
use iced::{
    Element, Task, Theme, application,
    widget::{button, column, horizontal_space, row, text},
};

mod icons;
mod mod_list;

fn main() -> iced::Result {
    application("Barnacle", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    ModList(mod_list::Message),
}

struct App {
    repo: Repository,
    theme: Theme,
    // Components
    mod_list: ModList,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let repo = Repository::new().unwrap();
        let (mod_list, task) = ModList::new(repo.clone());

        (
            Self {
                repo: repo.clone(),
                theme: Theme::Dark,
                mod_list,
            },
            task.map(Message::ModList),
        )
    }

    // Update application state based on messages passed by view()
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Redirect messages to relevant child components
            Message::ModList(msg) => match self.mod_list.update(msg) {
                mod_list::Action::Task(t) => t.map(Message::ModList),
                mod_list::Action::None => Task::none(),
            },
        }
    }

    // Render the application and pass along messages from components to update()
    fn view(&self) -> Element<'_, Message> {
        column![
            row![
                text("Game:"),
                button(icon("play")),
                text("Profile:"),
                horizontal_space(),
                button(icon("library")),
                button(icon("settings")),
                button(icon("notifications"))
            ],
            self.mod_list.view().map(Message::ModList)
        ]
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
