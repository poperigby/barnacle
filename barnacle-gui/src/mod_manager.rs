use iced::{Element, widget::column};

use crate::Message;

#[derive(Debug, Clone)]
pub enum ModManagerMessage {}

#[derive(Debug)]
pub struct ModManagerPage;

impl ModManagerPage {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: ModManagerMessage) {}

    pub fn view(&self) -> Element<'_, Message> {
        column!["Mod list!",].into()
    }
}
