use iced::{Element, widget::text};
use iced_aw::TabLabel;

use crate::library_manager::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded,
}

pub struct ToolsTab {}

impl ToolsTab {
    pub fn update(&mut self, message: Message) {}
}

impl Tab for ToolsTab {
    type Message = Message;

    fn title(&self) -> String {
        "Tools".into()
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        text("Tools").into()
    }
}
