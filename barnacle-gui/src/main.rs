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

#[derive(Debug)]
enum Page {
    ModManager(ModManagerPage),
    LibraryManager(LibraryManagerPage),
}

#[derive(Debug)]
struct App {
    theme: Theme,
    page: Page,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Dark,
                page: Page::ModManager(ModManagerPage::new()),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        match &self.page {
            Page::ModManager(p) => p.view(),
            Page::LibraryManager(p) => p.view(),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
