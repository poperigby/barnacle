use barnacle_lib::{Game, GameId, Mod, Profile, Repository};
use iced::{
    Element, Task,
    widget::{column, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(LibraryData),
}

pub enum State {
    Loading,
    Loaded(LibraryData),
    Error(String),
}

// Action the ModList wants the App to perform for it
pub enum Action {
    Task(Task<Message>),
    None,
}

#[derive(Debug, Clone)]
pub struct LibraryData {
    current_game: Option<GameId>,
    games: Vec<Game>,
    profiles: Vec<Profile>,
    mods: Vec<Mod>,
}

pub struct LibraryManager {
    repo: Repository,
    state: State,
}

impl LibraryManager {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        let task = Task::perform(
            {
                let repo = repo.clone();
                async move {
                    let games = repo.games().await.unwrap();
                    let current_game = games.first().and_then(|g| g.id());

                    LibraryData {
                        current_game,
                        games,
                        profiles: match current_game {
                            Some(id) => repo.profiles(id).await.unwrap(),
                            None => Vec::new(),
                        },
                        mods: match current_game {
                            Some(id) => repo.mods(id).await.unwrap(),
                            None => Vec::new(),
                        },
                    }
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
            Message::Loaded(data) => {
                self.state = State::Loaded(data);
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => column![text("Loading...")],
            State::Loaded(data) => column![text("Loaded!")],
            State::Error(e) => column![text(e)],
        }
        .into()
    }
}
