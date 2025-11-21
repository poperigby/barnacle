use barnacle_gui::Component;
use barnacle_lib::Repository;
use iced::{
    Element, Task,
    widget::{column, container, row, text, text_input},
};

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
}

pub struct NewDialog {
    name_content: String,
}

impl Component for NewDialog {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Self::Message>) {
        (
            Self {
                name_content: "".into(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NameInput(content) => {
                self.name_content = content;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        container(column![row![
            text("Name: "),
            text_input("Name", &self.name_content).on_input(Message::NameInput)
        ],])
        .padding(20)
        .width(400)
        .height(600)
        .style(container::rounded_box)
        .into()
    }
}
