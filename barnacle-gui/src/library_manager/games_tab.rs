use barnacle_lib::Repository;
use iced::{Element, widget::text};
use iced_aw::TabLabel;

use crate::library_manager::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded,
}

pub enum State {
    Loading,
    Loaded,
    Error(String),
}

pub struct GamesTab {
    // repo: Repository,
    // state: State,
}

impl GamesTab {
    pub fn update(&mut self, message: Message) {}
}

impl Tab for GamesTab {
    type Message = Message;

    fn title(&self) -> String {
        "Games".into()
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        text("Games").into()
    }
}
