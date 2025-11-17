use barnacle_lib::{ProfileMod, Repository};
use iced::{Element, Task, widget::text};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<ProfileMod>),
}

pub enum State {
    Loading,
    Loaded(Vec<ProfileMod>),
    Error(String),
}

// Action the ModList wants the App to perform for it
pub enum Action {
    Task(Task<Message>),
    None,
}

pub struct ModList {
    repo: Repository,
    state: State,
}

impl ModList {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
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

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Loaded(mods) => {
                self.state = State::Loaded(mods);
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => text("Loading..."),
            State::Loaded(mods) => text("Loaded..."),
            State::Error(e) => text(e),
        }
        .into()
    }
}
