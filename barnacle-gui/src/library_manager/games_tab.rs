use barnacle_lib::{Game, Repository};
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, horizontal_space, row, scrollable, text},
};
use iced_aw::TabLabel;

use crate::{
    icons::icon,
    library_manager::{TAB_PADDING, Tab},
    modal,
};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Game>),
    ShowNewDialog,
    HideNewDialog,
}

pub enum Action {
    Task(Task<Message>),
    None,
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
}

impl GamesTab {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                show_new_dialog: false,
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

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Loaded(games) => {
                self.state = State::Loaded(games);
                Action::None
            }
            Message::ShowNewDialog => {
                self.show_new_dialog = true;
                Action::None
            }
            Message::HideNewDialog => {
                self.show_new_dialog = false;
                Action::None
            }
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
                    modal(content, text("HOOP"), Message::HideNewDialog)
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
