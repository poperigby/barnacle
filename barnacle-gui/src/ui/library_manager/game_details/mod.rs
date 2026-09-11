use crate::{
    icons::Icon,
    model::{GameItem, Model, Mutation, ProfileItem},
};
use barnacle_gui::modal;
use barnacle_lib::{Game, repository::Profile};
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, space, text},
};

use crate::ui::library_manager::game_details::{edit_dialog::EditDialog, new_dialog::NewDialog};

pub mod edit_dialog;
pub mod new_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    ActivateGameButtonPressed(Game),
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
pub struct GameDetails {
    show_new_dialog: bool,

    // Children
    new_dialog: NewDialog,
    edit_dialog: EditDialog,
}

impl GameDetails {
    pub fn new() -> Self {
        let new_dialog = NewDialog::new();
        let edit_dialog = EditDialog::new();

        Self {
            show_new_dialog: false,

            new_dialog,
            edit_dialog,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ActivateGameButtonPressed(game) => {
                Action::Mutate(Mutation::ActivateGame(game))
            }
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
    pub fn view(&self, model: &Model, selected_game: &Option<GameItem>) -> Element<'_, Message> {
        let game = match selected_game {
            Some(game) => game,
            None => return column![text("No game selected")].into(),
        };

        let active_control: Element<'_, Message> = if game.active {
            text("Current").into()
        } else {
            button("Activate")
                .on_press(Message::ActivateGameButtonPressed(game.handle()))
                .into()
        };

        let top_row = row![text(game.name.clone()), space::horizontal(), active_control];

        let profile_rows: Vec<_> = game.profiles.iter().map(profile_row).collect();
        let profiles_view = column![text("Profiles"), scrollable(column(profile_rows))];

        let content = column![top_row, profiles_view]
            .width(Length::FillPortion(2))
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

fn profile_row<'a>(row: &ProfileItem) -> Element<'a, Message> {
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
