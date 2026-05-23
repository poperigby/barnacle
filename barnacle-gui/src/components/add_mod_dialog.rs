use std::env;

use fluent_i18n::t;
use iced::{
    Element, Task,
    widget::{button, column, container, row, space, text, text_input},
};
use rfd::AsyncFileDialog;

use crate::icons::icon;

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    PathChanged(String),
    PickPath(PickPathKind),
    PathPicked(Option<String>),
    CancelButtonPressed,
    AddButtonPressed,
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    AddMod { name: String, path: String },
    Cancel,
}

#[derive(Debug, Clone)]
pub enum PickPathKind {
    Archive,
    Directory,
}

pub struct AddModDialog {
    name: String,
    path: String,
}

impl AddModDialog {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                name: "".into(),
                path: "".into(),
            },
            Task::none(),
        )
    }

    fn clear(&mut self) {
        self.name.clear();
        self.path.clear();
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NameChanged(name) => {
                self.name = name;
                Action::None
            }
            Message::PathChanged(path) => {
                self.path = path;
                Action::None
            }
            Message::PickPath(kind) => Action::Run(Task::perform(
                async move {
                    let picker = AsyncFileDialog::new().set_directory(env::home_dir().unwrap());

                    match kind {
                        PickPathKind::Archive => {
                            picker
                                .add_filter("Archives", &["zip", "7z", "rar"])
                                .add_filter("All Files", &["*"])
                                .pick_file()
                                .await
                        }
                        PickPathKind::Directory => picker.pick_folder().await,
                    }
                    .map(|f| f.path().display().to_string())
                },
                Message::PathPicked,
            )),
            Message::PathPicked(path) => {
                if let Some(path) = path {
                    self.path = path;
                }
                Action::None
            }
            Message::CancelButtonPressed => {
                self.clear();
                Action::Cancel
            }
            Message::AddButtonPressed => Action::AddMod {
                name: self.name.clone(),
                path: self.path.clone(),
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![
            row![
                text(t!("name")),
                text_input("...", &self.name).on_input(Message::NameChanged)
            ],
            row![
                text(t!("path")),
                text_input("...", &self.path).on_input(Message::PathChanged),
                button(icon("archive")).on_press(Message::PickPath(PickPathKind::Archive)),
                button(icon("directory")).on_press(Message::PickPath(PickPathKind::Directory))
            ],
            space::vertical(),
            row![
                space::horizontal(),
                button(text(t!("cancel"))).on_press(Message::CancelButtonPressed),
                button(text(t!("add")))
                    .on_press_maybe(self.validate().then_some(Message::AddButtonPressed))
            ]
        ])
        .padding(20)
        .width(400)
        .height(600)
        .style(container::rounded_box)
        .into()
    }

    fn validate(&self) -> bool {
        !self.name.is_empty() && !self.path.is_empty()
    }
}
