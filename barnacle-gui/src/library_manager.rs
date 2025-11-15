use iced::{Element, Task, widget::column};

#[derive(Debug, Clone)]
pub enum LibraryManagerMessage {}

#[derive(Debug)]
pub struct LibraryManagerPage;

impl LibraryManagerPage {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: LibraryManagerMessage) -> Task<LibraryManagerMessage> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, LibraryManagerMessage> {
        column!["Mod list!",].into()
    }
}
