use barnacle_gui::icons::icon;
use barnacle_lib::{
    Repository,
    repository::{Game, Profile},
};
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, scrollable, space, text},
};

use crate::{
    components::library_manager::{
        TAB_PADDING,
        profiles_tab::{edit_dialog::EditDialog, new_dialog::NewDialog},
    },
    modal,
};

mod edit_dialog;
mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Profile>),
    ProfileDeleted,
    ShowNewDialog,
    ShowEditDialog(Profile),
    DeleteButtonPressed(Profile),
    // Child messages
    NewDialog(new_dialog::Message),
    EditDialog(edit_dialog::Message),
}

pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<Profile>),
}

pub struct Tab {
    selected_game: Game,
    state: State,
    show_new_dialog: bool,
    show_edit_dialog: bool,
    // Components
    new_dialog: NewDialog,
    edit_dialog: EditDialog,
}

impl Tab {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        let selected_game = repo.games().unwrap().pop().unwrap();

        let (new_dialog, _) = NewDialog::new(selected_game.clone());
        let (edit_dialog, _) = EditDialog::new();

        (
            Self {
                selected_game: selected_game.clone(),
                state: State::Loading,
                show_new_dialog: false,
                show_edit_dialog: false,
                new_dialog,
                edit_dialog,
            },
            update_profiles_list(&selected_game.clone()),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // State
            Message::Loaded(profiles) => {
                self.state = State::Loaded(profiles);
                Task::none()
            }
            Message::ProfileDeleted => update_profiles_list(&self.selected_game),
            // Components
            Message::ShowNewDialog => {
                self.show_new_dialog = true;
                Task::none()
            }
            Message::ShowEditDialog(profile) => {
                self.edit_dialog.load(profile);
                self.show_edit_dialog = true;
                Task::none()
            }
            Message::DeleteButtonPressed(profile) => Task::perform(
                {
                    // So we don't try to query deleted profiles
                    self.state = State::Loading;

                    let mut game = self.selected_game.clone();
                    async move { game.remove_profile(profile).unwrap() }
                },
                |_| Message::ProfileDeleted,
            ),
            Message::NewDialog(msg) => match msg {
                new_dialog::Message::CancelPressed => {
                    self.show_new_dialog = false;
                    self.new_dialog.clear();
                    Task::none()
                }
                new_dialog::Message::ProfileCreated => {
                    self.state = State::Loading;
                    self.show_new_dialog = false;
                    update_profiles_list(&self.selected_game)
                }
                _ => self.new_dialog.update(msg).map(Message::NewDialog),
            },
            Message::EditDialog(msg) => match msg {
                edit_dialog::Message::CancelPressed => {
                    self.show_edit_dialog = false;
                    Task::none()
                }
                edit_dialog::Message::ProfileEdited => {
                    self.show_edit_dialog = false;
                    Task::none()
                }
                _ => self.edit_dialog.update(msg).map(Message::EditDialog),
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => column![text("Loading...")].into(),
            State::Error(_e) => column![text("ERROR!")].into(),
            State::Loaded(profiles) => {
                let children = profiles.iter().map(profile_row);

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

fn update_profiles_list(selected_game: &Game) -> Task<Message> {
    Task::perform(
        {
            let game = selected_game.clone();
            async move { game.profiles().unwrap() }
        },
        Message::Loaded,
    )
}

fn profile_row<'a>(profile: &Profile) -> Element<'a, Message> {
    container(
        row![
            text(profile.name().unwrap()),
            space::horizontal(),
            button(icon("edit")).on_press(Message::ShowEditDialog(profile.clone())),
            button(icon("delete")).on_press(Message::DeleteButtonPressed(profile.clone()))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
