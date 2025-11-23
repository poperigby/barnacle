use crate::components::{
    library_manager::{self, LibraryManager},
    mod_list::{self, ModList},
};
use barnacle_gui::{Component, icons::icon, modal};
use barnacle_lib::Repository;
use iced::{
    Element,
    Length::Fill,
    Task, Theme, application,
    widget::{button, column, horizontal_space, row, text},
};

mod components;

fn main() -> iced::Result {
    application(App::title, App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
    ShowLibraryManager,
    HideLibraryManager,
}

struct App {
    title: String,
    repo: Repository,
    theme: Theme,
    // Components
    mod_list: ModList,
    library_manager: LibraryManager,
    show_library_manager: bool,
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
                show_library_manager: false,
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
            Message::ModList(msg) => self.mod_list.update(msg).map(Message::ModList),
            Message::LibraryManager(msg) => self
                .library_manager
                .update(msg)
                .map(Message::LibraryManager),
            Message::ShowLibraryManager => {
                self.show_library_manager = true;
                Task::none()
            }
            Message::HideLibraryManager => {
                self.show_library_manager = false;
                Task::none()
            }
        }
    }

    // Render the application and pass along messages from components to update()
    fn view(&self) -> Element<'_, Message> {
        let content = column![
            // Top bar
            row![
                text("Game:"),
                button(icon("play")),
                text("Profile:"),
                horizontal_space(),
                button(icon("library")).on_press(Message::ShowLibraryManager),
                button(icon("settings")),
                button(icon("notifications"))
            ],
            // Action bar
            row![],
            // Mod list
            self.mod_list.view().map(Message::ModList),
        ]
        .height(Fill);

        if self.show_library_manager {
            modal(
                content,
                self.library_manager.view().map(Message::LibraryManager),
                Message::HideLibraryManager,
            )
        } else {
            content.into()
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
