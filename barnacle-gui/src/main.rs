use crate::{icons::icon, library_manager::LibraryManager, mod_list::ModList};
use barnacle_lib::Repository;
use iced::{
    Element, Task, Theme, application,
    widget::{button, column, horizontal_space, row, text},
};

mod icons;
mod library_manager;
mod mod_list;

fn main() -> iced::Result {
    application(App::title, App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
}

struct App {
    title: String,
    repo: Repository,
    theme: Theme,
    // Components
    mod_list: ModList,
    library_manager: LibraryManager,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let repo = Repository::new().unwrap();
        let (mod_list, mod_list_task) = ModList::new(repo.clone());
        let (library_manager, library_manager_task) = LibraryManager::new(repo.clone());

        (
            Self {
                title: "Barnacle".into(),
                repo: repo.clone(),
                theme: Theme::Dark,
                mod_list,
                library_manager,
            },
            Task::batch([
                mod_list_task.map(Message::ModList),
                library_manager_task.map(Message::LibraryManager),
            ]),
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
            Message::LibraryManager(msg) => match self.library_manager.update(msg) {
                library_manager::Action::Task(t) => t.map(Message::LibraryManager),
                library_manager::Action::None => Task::none(),
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
            self.mod_list.view().map(Message::ModList),
            self.library_manager.view().map(Message::LibraryManager)
        ]
        .into()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
