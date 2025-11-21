use barnacle_gui::{Component, icons::icon};
use barnacle_lib::{Game, Repository};
use iced::{
    Element, Length, Task,
    widget::{
        Column, button, column, container, horizontal_space, row, scrollable, text, text_input,
    },
};

use crate::{
    components::library_manager::{TAB_PADDING, games_tab::new_dialog::NewDialog},
    modal,
};

mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Game>),
    ShowNewDialog,
    HideNewDialog,
    // Components
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
    new_dialog: NewDialog,
}

impl Component for Tab {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let (new_dialog, new_dialog_task) = NewDialog::new(repo.clone());

        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                show_new_dialog: false,
                new_dialog,
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
            Message::Loaded(games) => {
                self.state = State::Loaded(games);
                Task::none()
            }
            Message::ShowNewDialog => {
                self.show_new_dialog = true;
                Task::none()
            }
            Message::HideNewDialog => {
                self.show_new_dialog = false;
                Task::none()
            }
            Message::NewDialog(msg) => self.new_dialog.update(msg).map(Message::NewDialog),
        }
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
