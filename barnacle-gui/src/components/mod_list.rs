use barnacle_gui::Component;
use barnacle_lib::{ProfileMod, Repository};
use iced::{
    Element, Length, Task,
    widget::{column, scrollable, table, text},
};

#[derive(Debug, Clone)]
pub struct ModRow {
    name: String,
    notes: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<ProfileMod>),
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<ModRow>),
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
                let mut repo = repo.clone();
                async move {
                    if repo.games().await.unwrap().is_empty() {
                        let game_id = repo
                            .add_game("Skyrim", barnacle_lib::DeployKind::CreationEngine)
                            .await
                            .unwrap();
                        let profile_id = repo.add_profile(game_id, "Test").await.unwrap();

                        repo.set_current_profile(profile_id).await.unwrap();

                        let mod_id = repo.add_mod(game_id, None, "Test").await.unwrap();
                        repo.add_mod_entry(mod_id, profile_id).await.unwrap();
                    }

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
            Message::Loaded(mods) => {
                let rows = mods
                    .iter()
                    .map(|m| ModRow {
                        name: m.data().name().to_string(),
                        notes: m.entry().notes().to_string(),
                    })
                    .collect();

                self.state = State::Loaded(rows)
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => column![text("Loading mods...")],
            State::Error(e) => column![text(e)],
            State::Loaded(rows) => {
                let columns = [
                    table::column(text("Name"), |row: ModRow| text(row.name)),
                    table::column(text("Notes"), |row: ModRow| text(row.notes)),
                ];

                column![scrollable(table(columns, rows.clone()).width(Length::Fill))]
            }
        }
        .into()
    }
}
