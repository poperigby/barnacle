use barnacle_gui::{config::GuiConfig, icons::icon, modal};
use barnacle_lib::Repository;
use iced::{
    Element,
    Length::Fill,
    Task, Theme, application,
    widget::{button, column, row, space, text},
};
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::components::{
    library_manager::{self, LibraryManager},
    mod_list::{self, ModList},
};

mod components;

fn main() -> iced::Result {
    application(App::new, App::update, App::view)
        .theme(App::theme)
        .title(App::title)
        .run()
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
    theme: Theme,
    // Components
    mod_list: ModList,
    library_manager: LibraryManager,
    show_library_manager: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        // Human friendly panicking in release mode
        human_panic::setup_panic!();

        // Logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let repo = Repository::new();
        let cfg = GuiConfig::load();
        let theme = cfg.theme();

        let (mod_list, mod_list_task) = ModList::new(repo.clone());
        let (library_manager, library_manager_task) = LibraryManager::new(repo.clone());

        (
            Self {
                title: "Barnacle".into(),
                theme,
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
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Redirect messages to relevant child components
            Message::ModList(msg) => self.mod_list.update(msg).map(Message::ModList),
            Message::LibraryManager(msg) => match msg {
                library_manager::Message::CloseButtonSelected => {
                    self.show_library_manager = false;
                    Task::none()
                }
                _ => self
                    .library_manager
                    .update(msg)
                    .map(Message::LibraryManager),
            },
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
    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            // Top bar
            row![
                text("Game:"),
                button(icon("play")),
                text("Profile:"),
                space::horizontal(),
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
                None,
            )
        } else {
            content.into()
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
