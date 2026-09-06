use fluent_i18n::t;
use iced::{
    Element, Task,
    widget::{button, column, container, row, space, text, text_input},
};

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    CancelPressed,
    CreatePressed,
}

pub enum Action {
    None,
    Run(Task<Message>),
    Create(NewProfile),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct NewDialog {
    name: String,
}

#[derive(Debug, Clone)]
pub struct NewProfile {
    pub name: String,
}

impl NewDialog {
    pub fn new() -> (Self, Task<Message>) {
        (Self { name: "".into() }, Task::none())
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
            Message::CancelPressed => {
                self.clear();
                Action::Cancel
            }
            Message::CreatePressed => {
                let name = self.name.clone();

                self.clear();

                Action::Create(NewProfile { name })
            }
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
                button(text(t!("create")))
                    .on_press_maybe(self.validate().then_some(Message::CreatePressed)),
            ],
        ])
        .padding(20)
        .into()
    }

    fn validate(&self) -> bool {
        !self.name.is_empty()
    }
}
