use barnacle_lib::repository::Profile;
use iced::{
    Element, Task,
    widget::{button, column, container, row, space, text, text_input},
};

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    CancelPressed,
    ConfirmPressed,
    ProfileEdited,
}

pub struct EditDialog {
    profile: Option<Profile>,
    name: String,
}

impl EditDialog {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                profile: None,
                name: "".into(),
            },
            Task::none(),
        )
    }

    /// Load a new [`Profile`] for editing.
    pub fn load(&mut self, profile: Profile) {
        self.profile = Some(profile.clone());

        self.name = profile.name().unwrap();
    }

    /// Reset the dialog state
    pub fn clear(&mut self) {
        self.name.clear();
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NameInput(content) => {
                self.name = content;
                Task::none()
            }
            Message::CancelPressed => Task::none(),
            Message::ConfirmPressed => {
                let mut profile = self.profile.clone();

                let new_name = self.name.clone();

                self.clear();

                Task::perform(
                    async move {
                        profile.as_mut().unwrap().set_name(&new_name).unwrap();
                    },
                    |_| Message::ProfileEdited,
                )
            }
            Message::ProfileEdited => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![
            row![
                text("Name: "),
                text_input("Name", &self.name).on_input(Message::NameInput),
            ],
            space::vertical(),
            row![
                space::horizontal(),
                button("Cancel").on_press(Message::CancelPressed),
                button("Confirm").on_press(Message::ConfirmPressed),
            ],
        ])
        .padding(20)
        .width(400)
        .height(600)
        .style(container::rounded_box)
        .into()
    }
}
