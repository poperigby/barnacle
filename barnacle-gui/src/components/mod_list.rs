use barnacle_gui::{Component, icons::icon};
use barnacle_lib::{ModId, ProfileMod, Repository};
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, space, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<ProfileMod>),
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<ProfileMod>),
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
            Message::Loaded(mods) => self.state = State::Loaded(mods),
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => column![text("Loading mods...")],
            State::Error(e) => column![text(e)],
            State::Loaded(mods) => {
                let rows = mods.iter().map(|m| mod_row(m.data().name()));

                Column::with_children(rows)
            }
        }
        .into()
    }
}

fn mod_row<'a>(name: &'a str) -> Element<'a, Message> {
    container(
        row![
            text(name),
            space::horizontal(),
            button(icon("edit")),
            button(icon("delete"))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
