use barnacle_lib::db::Database;
use iced::{Element, Task, Theme, application};

use crate::{
    library_manager::{LibraryManagerMessage, LibraryManagerPage},
    mod_manager::{ModManagerMessage, ModManagerPage},
};

mod library_manager;
mod mod_manager;

fn main() -> iced::Result {
    application("Barnacle", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    ModManager(ModManagerMessage),
    LibraryManager(LibraryManagerMessage),
}

enum Page {
    ModManager(ModManagerPage),
    LibraryManager(LibraryManagerPage),
}

struct App {
    db: Database,
    theme: Theme,
    page: Page,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let db = Database::new().unwrap();

        (
            Self {
                db: db.clone(),
                theme: Theme::Dark,
                page: Page::ModManager(ModManagerPage::new(db.clone())),
            },
            Task::none(),
        )
    }

    // Update application state based on messages passed by view()
    fn update(&mut self, message: Message) -> Task<Message> {
        match (&mut self.page, message) {
            // Route page specific messages to their handler
            (Page::ModManager(p), Message::ModManager(msg)) => {
                p.update(msg).map(Message::ModManager)
            }
            (Page::LibraryManager(p), Message::LibraryManager(msg)) => {
                p.update(msg).map(Message::LibraryManager)
            }
            _ => Task::none(),
        }
    }

    // Render the application and pass along messages from components to update()
    fn view(&self) -> Element<'_, Message> {
        match &self.page {
            Page::ModManager(p) => p.view().map(Message::ModManager),
            Page::LibraryManager(p) => p.view().map(Message::LibraryManager),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
