use barnacle_gui::{Component, icons::icon};
use barnacle_lib::{Game, Repository};
use iced::{
    Element, Length, Task,
    widget::{
        Column, button, column, container, horizontal_space, row, scrollable, text, text_input,
    },
};

use crate::{components::library_manager::TAB_PADDING, modal};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Game>),
    ShowNewDialog,
    HideNewDialog,
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<Game>),
}

pub struct GamesTab {
    repo: Repository,
    state: State,
    show_new_dialog: bool,
    new_dialog_name: String,
}

impl Component for GamesTab {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                show_new_dialog: false,
                new_dialog_name: "".into(),
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(games) => self.state = State::Loaded(games),
            Message::ShowNewDialog => self.show_new_dialog = true,
            Message::HideNewDialog => self.show_new_dialog = false,
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.state {
            State::Loading => column![text("Loading...")].into(),
            State::Error(e) => column![text("ERROR!")].into(),
            State::Loaded(games) => {
                let children = games.iter().map(|g| game_row(g.name()));

                let content = column![
                    row![button("New").on_press(Message::ShowNewDialog)],
                    scrollable(Column::with_children(children)).width(Length::Fill)
                ]
                .padding(TAB_PADDING);

                if self.show_new_dialog {
                    let new_dialog = container(column![row![
                        text("Name: "),
                        text_input("Name", &self.new_dialog_name),
                    ],])
                    .padding(20)
                    .width(400)
                    .height(600)
                    .style(container::rounded_box);

                    modal(content, new_dialog, Message::HideNewDialog)
                } else {
                    content.into()
                }
            }
        }
    }
}

fn game_row<'a>(name: &'a str) -> Element<'a, Message> {
    container(
        row![
            text(name),
            horizontal_space(),
            button(icon("edit")),
            button(icon("delete"))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
