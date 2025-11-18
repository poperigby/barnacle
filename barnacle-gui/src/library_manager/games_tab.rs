use barnacle_lib::{Game, Repository};
use iced::{Element, Task, widget::text};
use iced_aw::TabLabel;

use crate::library_manager::Tab;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Game>),
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<Game>),
}

pub struct GamesTab {
    repo: Repository,
    state: State,
}

impl GamesTab {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
            },
            Task::perform(
                {
                    let repo = repo.clone();
                    async move { repo.games().await.unwrap() }
                },
                Message::Loaded,
            ),
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Loaded(games) => self.state = State::Loaded(games),
        }
    }
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
        match &self.state {
            State::Loading => text("Loading..."),
            State::Error(e) => text("ERROR!"),
            State::Loaded(games) => text("Loaded!"),
        }
        .into()
    }
}
