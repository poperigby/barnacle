use barnacle_gui::{Component, icons::icon};
use barnacle_lib::{Repository, repository::Game};
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, scrollable, space, text},
};

use crate::{
    components::library_manager::{
        TAB_PADDING,
        games_tab::{edit_dialog::EditDialog, new_dialog::NewDialog},
    },
    modal,
};

mod edit_dialog;
mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Game>),
    GameDeleted,
    ShowNewDialog,
    HideNewDialog,
    ShowEditDialog(Game),
    HideEditDialog,
    DeleteButtonPressed(Game),
    // Child messages
    NewDialog(new_dialog::Message),
    EditDialog(edit_dialog::Message),
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
    show_edit_dialog: bool,
    // Components
    new_dialog: NewDialog,
    edit_dialog: EditDialog,
}

impl Component for Tab {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let (new_dialog, _) = NewDialog::new(repo.clone());
        let (edit_dialog, _) = EditDialog::new();

        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                show_new_dialog: false,
                show_edit_dialog: false,
                new_dialog,
                edit_dialog,
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
            Message::GameDeleted => update_games_list(&self.repo),
            // Components
            Message::ShowNewDialog => {
                self.show_new_dialog = true;
                Task::none()
            }
            Message::HideNewDialog => {
                self.show_new_dialog = false;
                Task::none()
            }
            Message::ShowEditDialog(game) => {
                self.edit_dialog.load(game);
                self.show_edit_dialog = true;
                Task::none()
            }
            Message::HideEditDialog => {
                self.show_edit_dialog = false;
                Task::none()
            }
            Message::DeleteButtonPressed(game) => Task::perform(
                {
                    // So we don't try to query deleted games
                    self.state = State::Loading;

                    let repo = self.repo.clone();
                    async move { repo.remove_game(game).unwrap() }
                },
                |_| Message::GameDeleted,
            ),
            Message::NewDialog(msg) => match msg {
                new_dialog::Message::CancelPressed => {
                    self.show_new_dialog = false;
                    self.new_dialog.clear();
                    Task::none()
                }
                new_dialog::Message::GameCreated => {
                    self.state = State::Loading;
                    self.show_new_dialog = false;
                    update_games_list(&self.repo)
                }
                _ => self.new_dialog.update(msg).map(Message::NewDialog),
            },
            Message::EditDialog(msg) => match msg {
                edit_dialog::Message::CancelPressed => {
                    self.show_edit_dialog = false;
                    Task::none()
                }
                edit_dialog::Message::GameEdited => {
                    self.show_edit_dialog = false;
                    Task::none()
                }
                _ => self.edit_dialog.update(msg).map(Message::EditDialog),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.state {
            State::Loading => column![text("Loading...")].into(),
            State::Error(_e) => column![text("ERROR!")].into(),
            State::Loaded(games) => {
                let children = games.iter().map(game_row);

                let content = column![
                    row![button("New").on_press(Message::ShowNewDialog)],
                    scrollable(Column::with_children(children)).width(Length::Fill)
                ]
                .padding(TAB_PADDING);

                if self.show_new_dialog {
                    modal(
                        content,
                        self.new_dialog.view().map(Message::NewDialog),
                        None,
                    )
                } else if self.show_edit_dialog {
                    modal(
                        content,
                        self.edit_dialog.view().map(Message::EditDialog),
                        None,
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
            async move { repo.games().unwrap() }
        },
        Message::Loaded,
    )
}

fn game_row<'a>(game: &Game) -> Element<'a, Message> {
    container(
        row![
            text(game.name().unwrap()),
            space::horizontal(),
            button(icon("edit")).on_press(Message::ShowEditDialog(game.clone())),
            button(icon("delete")).on_press(Message::DeleteButtonPressed(game.clone()))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
