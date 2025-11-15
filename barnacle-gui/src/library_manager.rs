use iced::{Element, widget::column};

use crate::Message;

#[derive(Debug, Clone)]
pub enum LibraryManagerMessage {}

#[derive(Debug)]
pub struct LibraryManagerPage;

impl LibraryManagerPage {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: LibraryManagerMessage) {}

    pub fn view(&self) -> Element<'_, Message> {
        column!["Mod list!",].into()
    }
}
