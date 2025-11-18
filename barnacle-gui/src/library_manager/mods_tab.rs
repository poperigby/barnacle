use iced::{Element, widget::text};
use iced_aw::TabLabel;

use crate::library_manager::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded,
}

pub struct ModsTab {}

impl ModsTab {
    pub fn update(&mut self, message: Message) {}
}

impl Tab for ModsTab {
    type Message = Message;

    fn title(&self) -> String {
        "Mods".into()
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        text("Mods").into()
    }
}
