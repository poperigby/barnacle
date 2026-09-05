use crate::{icons::icon, modal, workspace::library_manager::profiles_tab::new_dialog::NewProfile};
use barnacle_lib::{
    Repository,
    repository::{Game, Profile},
};
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, scrollable, space, text},
};
use iced_aw::Spinner;

use crate::workspace::library_manager::profiles_tab::{
    edit_dialog::EditDialog, new_dialog::NewDialog,
};

pub mod edit_dialog;
pub mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    StateChanged(State),
    NewButtonPressed,
    EditButtonPressed(Profile),
    EditLoaded { profile: Profile, name: String },
    DeleteButtonPressed(Profile),
    ProfileCreated,
    ProfileEdited,
    // Child messages
    NewDialog(new_dialog::Message),
    EditDialog(edit_dialog::Message),
}

pub enum Action {
    None,
    Run(Task<Message>),
    Refresh,
    Create(NewProfile),
    Delete(Profile),
}

#[derive(Debug, Clone)]
pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<ProfileRow>),
}

#[derive(Debug, Clone)]
pub struct Tab {
    repo: Repository,
    state: State,

    show_new_dialog: bool,

    // Children
    new_dialog: NewDialog,
    edit_dialog: EditDialog,
}

impl Tab {
    pub fn new(repo: Repository) -> Self {
        let (new_dialog, _) = NewDialog::new();
        let (edit_dialog, _) = EditDialog::new();

        Self {
            repo: repo.clone(),
            state: State::Loading,

            show_new_dialog: false,

            // Widget state
            new_dialog,
            edit_dialog,
        }
    }

    pub fn refresh(&self, game: &Game) -> Task<Message> {
        let game = game.clone();
        Task::perform(
            async move {
                let mut rows = Vec::new();
                for profile in game.profiles().await.unwrap() {
                    let name = profile.name().await.unwrap();
                    rows.push(ProfileRow::new(profile, name));
                }
                State::Loaded(rows)
            },
            Message::StateChanged,
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::StateChanged(state) => {
                self.state = state;
                Action::None
            }
            Message::ProfileCreated => Action::Refresh,
            Message::ProfileEdited => Action::Refresh,
            Message::NewButtonPressed => {
                self.show_new_dialog = true;
                Action::None
            }
            Message::EditButtonPressed(profile) => Action::Run(Task::perform(
                async move {
                    let name = profile.name().await.unwrap();
                    (profile, name)
                },
                |(profile, name)| Message::EditLoaded { profile, name },
            )),
            Message::EditLoaded { profile, name } => {
                self.edit_dialog.load(profile, name);
                Action::None
            }
            Message::DeleteButtonPressed(profile) => {
                self.state = State::Loading;
                Action::Delete(profile)
            }
            Message::NewDialog(message) => match self.new_dialog.update(message) {
                new_dialog::Action::None => Action::None,
                new_dialog::Action::Run(task) => Action::Run(task.map(Message::NewDialog)),
                new_dialog::Action::Create(new_profile) => {
                    self.state = State::Loading;
                    self.show_new_dialog = false;
                    Action::Create(new_profile)
                }
                new_dialog::Action::Cancel => {
                    self.show_new_dialog = false;
                    Action::None
                }
            },
            Message::EditDialog(message) => match &self.state {
                State::Loaded { .. } => match self.edit_dialog.update(message) {
                    edit_dialog::Action::None => Action::None,
                    edit_dialog::Action::Run(task) => Action::Run(task.map(Message::EditDialog)),
                    edit_dialog::Action::Cancel => Action::None,
                    edit_dialog::Action::Edit { profile, name } => Action::Run(Task::perform(
                        async move {
                            profile.set_name(&name).await.unwrap();
                        },
                        |_| Message::ProfileEdited,
                    )),
                },
                _ => Action::None,
            },
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let content = match &self.state {
            State::Loading => Spinner::new().into(),
            State::Error(e) => text(e).into(),
            State::Loaded(profile_rows) => column![
                button(text(t!("new"))).on_press(Message::NewButtonPressed),
                scrollable(Column::with_children(profile_rows.iter().map(|p| p.view())))
            ]
            .into(),
        };

        if self.show_new_dialog {
            modal(
                content,
                self.new_dialog.view().map(Message::NewDialog),
                None,
            )
        } else {
            content
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileRow {
    profile: Profile,
    name: String,
}

impl ProfileRow {
    pub fn new(profile: Profile, name: String) -> Self {
        Self { profile, name }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        container(
            row![
                text(self.name.clone()),
                space::horizontal(),
                button(icon("edit")),
                button(icon("delete")).on_press(Message::DeleteButtonPressed(self.profile.clone()))
            ]
            .padding(12),
        )
        .width(Length::Fill)
        .style(container::bordered_box)
        .into()
    }
}
