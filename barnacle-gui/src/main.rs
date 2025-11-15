use iced::{Element, Task, application, widget::column};

fn main() -> iced::Result {
    application("Barnacle", App::update, App::view).run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    ModList,
    LibraryManager,
}

#[derive(Default, Debug)]
enum Page {
    #[default]
    ModList,
    LibraryManager,
}

#[derive(Default, Debug)]
struct App {
    page: Page,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        match self.page {
            Page::ModList => column!["Mod list!",],
            Page::LibraryManager => column!["Library manager!"],
        }
        .into()
    }
}
