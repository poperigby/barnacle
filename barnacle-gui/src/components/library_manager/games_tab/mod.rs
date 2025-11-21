use barnacle_gui::{Component, icons::icon};
use barnacle_lib::{Game, GameId, Repository};
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, horizontal_space, row, scrollable, text},
};

use crate::{
    components::library_manager::{TAB_PADDING, games_tab::new_dialog::NewDialog},
    modal,
};

mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    // State
    Loaded(Vec<Game>),
    GameDeleted(GameId),
    // Components
    ShowNewDialog,
    HideNewDialog,
    DeleteButtonPressed(GameId),
    NewDialog(new_dialog::Message),
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<Game>),
}

pub struct Tab {
    repo: Repository,
    state: State,
    show_new_dialog: bool,
    // Components
    new_dialog: NewDialog,
}

impl Component for Tab {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let (new_dialog, _) = NewDialog::new(repo.clone());

        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                show_new_dialog: false,
                new_dialog,
            },
            update_games_list(&repo),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // State
            Message::Loaded(games) => {
                self.state = State::Loaded(games);
                Task::none()
            }
            Message::GameDeleted(_) => update_games_list(&self.repo),
            // Components
            Message::ShowNewDialog => {
                self.show_new_dialog = true;
                Task::none()
            }
            Message::HideNewDialog => {
                self.show_new_dialog = false;
                Task::none()
            }
            Message::DeleteButtonPressed(id) => Task::perform(
                {
                    let mut repo = self.repo.clone();
                    async move {
                        repo.delete_game(id).await.unwrap();
                        id
                    }
                },
                Message::GameDeleted,
            ),
            Message::NewDialog(msg) => match msg {
                new_dialog::Message::CancelPressed => {
                    self.show_new_dialog = false;
                    Task::none()
                }
                new_dialog::Message::GameCreated => {
                    self.show_new_dialog = false;
                    update_games_list(&self.repo)
                }
                _ => self.new_dialog.update(msg).map(Message::NewDialog),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.state {
            State::Loading => column![text("Loading...")].into(),
            State::Error(e) => column![text("ERROR!")].into(),
            State::Loaded(games) => {
                let children = games.iter().map(|g| game_row(g));

                let content = column![
                    row![button("New").on_press(Message::ShowNewDialog)],
                    scrollable(Column::with_children(children)).width(Length::Fill)
                ]
                .padding(TAB_PADDING);

                if self.show_new_dialog {
                    modal(
                        content,
                        self.new_dialog.view().map(Message::NewDialog),
                        Message::HideNewDialog,
                    )
                } else {
                    content.into()
                }
            }
        }
    }
}

fn update_games_list(repo: &Repository) -> Task<Message> {
    Task::perform(
        {
            let repo = repo.clone();
            async move { repo.games().await.unwrap() }
        },
        Message::Loaded,
    )
}

fn game_row(game: &'_ Game) -> Element<'_, Message> {
    container(
        row![
            text(game.name()),
            horizontal_space(),
            button(icon("edit")),
            button(icon("delete")).on_press(Message::DeleteButtonPressed(game.id().unwrap()))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
