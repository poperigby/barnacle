use barnacle_lib::{ProfileMod, Repository};
use iced::{
    Element, Task,
    widget::{Column, column, text},
};

use crate::Component;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<ProfileMod>),
}

pub enum State {
    Loading,
    Loaded(Vec<ProfileMod>),
    Error(String),
}

pub struct ModList {
    repo: Repository,
    state: State,
}

impl Component for ModList {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let task = Task::perform(
            {
                let repo = repo.clone();
                async move {
                    let current_profile = repo.clone().current_profile().await.unwrap().unwrap();

                    repo.profile_mods(current_profile).await.unwrap()
                }
            },
            Message::Loaded,
        );

        (
            Self {
                repo,
                state: State::Loading,
            },
            task,
        )
    }

    fn update(&mut self, message: Message) -> Task<Self::Message> {
        match message {
            Message::Loaded(mods) => self.state = State::Loaded(mods),
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => column![text("Loading...")],
            State::Loaded(mods) => column![text("Loaded!")],
            State::Error(e) => column![text(e)],
        }
        .into()
    }
}
