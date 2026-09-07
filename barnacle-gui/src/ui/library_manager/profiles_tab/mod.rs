use crate::{
    icons::Icon,
    model::{Model, Mutation, ProfileRow},
};
use barnacle_gui::modal;
use barnacle_lib::repository::Profile;
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, scrollable, space, text},
};

use crate::ui::library_manager::profiles_tab::{edit_dialog::EditDialog, new_dialog::NewDialog};

pub mod edit_dialog;
pub mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    NewButtonPressed,
    EditButtonPressed {
        profile: Profile,
        // The name the [`Profile`] has when the edit dialog is opened
        initial_name: String,
    },
    DeleteButtonPressed(Profile),

    // Children
    NewDialog(new_dialog::Message),
    EditDialog(edit_dialog::Message),
}

pub enum Action {
    None,
    Run(Task<Message>),
    Create { name: String },
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct Tab {
    show_new_dialog: bool,

    // Children
    new_dialog: NewDialog,
    edit_dialog: EditDialog,
}

impl Tab {
    pub fn new() -> Self {
        let new_dialog = NewDialog::new();
        let edit_dialog = EditDialog::new();

        Self {
            show_new_dialog: false,

            // Widget state
            new_dialog,
            edit_dialog,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NewButtonPressed => {
                self.show_new_dialog = true;

                Action::None
            }
            Message::EditButtonPressed {
                profile,
                initial_name: name,
            } => {
                self.edit_dialog.open(profile, name);

                Action::None
            }
            Message::DeleteButtonPressed(profile) => {
                Action::Mutate(Mutation::DeleteProfile(profile))
            }

            // Children
            Message::NewDialog(message) => match self.new_dialog.update(message) {
                new_dialog::Action::None => Action::None,
                new_dialog::Action::Run(task) => Action::Run(task.map(Message::NewDialog)),
                new_dialog::Action::Create { name } => {
                    self.show_new_dialog = false;

                    Action::Create { name }
                }
                new_dialog::Action::Cancel => {
                    self.show_new_dialog = false;

                    Action::None
                }
            },
            Message::EditDialog(message) => match self.edit_dialog.update(message) {
                edit_dialog::Action::None => Action::None,
                edit_dialog::Action::Run(task) => Action::Run(task.map(Message::EditDialog)),
                edit_dialog::Action::Mutate(mutation) => Action::Mutate(mutation),
                edit_dialog::Action::Cancel => Action::None,
            },
        }
    }
    pub fn view(&self, model: &Model) -> Element<'_, Message> {
        let content = column![
            button(text(t!("new"))).on_press(Message::NewButtonPressed),
            scrollable(Column::with_children(
                model.profiles().iter().map(profile_row)
            ))
        ]
        .into();

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

fn profile_row<'a>(row: &ProfileRow) -> Element<'a, Message> {
    container(
        row![
            text(row.name.clone()),
            space::horizontal(),
            button(Icon::Edit),
            button(Icon::Delete).on_press(Message::DeleteButtonPressed(row.handle().clone()))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
