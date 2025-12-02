use barnacle_gui::Component;
use barnacle_lib::{Repository, repository::entities::ModEntry};
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
    Loaded(Vec<ModEntry>),
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
                let repo = repo.clone();
                async move {
                    if repo.games().unwrap().is_empty() {
                        let mut game = repo
                            .add_game(
                                "Skyrim",
                                barnacle_lib::repository::DeployKind::CreationEngine,
                            )
                            .unwrap();
                        let mut profile = game.add_profile("Test").unwrap();

                        repo.set_current_profile(&profile).unwrap();

                        let mod_ = game.add_mod("Test", None).unwrap();
                        profile.add_mod_entry(mod_).unwrap();
                    }

                    let current_profile = repo.clone().current_profile().unwrap();
                    current_profile.mod_entries().unwrap()
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
                        name: m.name().unwrap().to_string(),
                        notes: m.notes().unwrap().to_string(),
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
