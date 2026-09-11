use barnacle_lib::repository::Profile;
use fluent_i18n::t;
use iced::{
    Element, Task,
    widget::{button, column, container, row, space, text, text_input},
};

use barnacle_gui::model::Mutation;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    CancelPressed,
    ConfirmPressed,
}

pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct EditDialog {
    profile: Option<Profile>,
    name: String,
}

impl EditDialog {
    pub fn new() -> Self {
        Self {
            profile: None,
            name: "".into(),
        }
    }

    /// Load a new [`Profile`] for editing.
    pub fn open(&mut self, profile: Profile, initial_name: String) {
        self.profile = Some(profile.clone());
        self.name = initial_name;
    }

    /// Reset the dialog state
    pub fn clear(&mut self) {
        self.name.clear();
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NameInput(content) => {
                self.name = content;
                Action::None
            }
            Message::CancelPressed => Action::Cancel,
            Message::ConfirmPressed => match &self.profile {
                Some(profile) => {
                    let profile = profile.clone();
                    let name = self.name.clone();

                    self.clear();

                    Action::Mutate(Mutation::EditProfile {
                        profile,
                        new_name: name,
                    })
                }
                None => Action::None,
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![
            row![
                text(t!("name")),
                text_input("...", &self.name).on_input(Message::NameInput),
            ],
            space::vertical(),
            row![
                space::horizontal(),
                button(text(t!("cancel"))).on_press(Message::CancelPressed),
                button(text(t!("confirm"))).on_press(Message::ConfirmPressed),
            ],
        ])
        .padding(20)
        .width(400)
        .height(600)
        .into()
    }
}
