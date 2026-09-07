use std::{env, path::PathBuf};

use fluent_i18n::t;
use iced::{
    Element, Task,
    widget::{button, column, container, row, space, text, text_input},
};
use rfd::AsyncFileDialog;

use crate::icons::Icon;

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    PathChanged(String),
    PickPath(PickPathKind),
    PathPicked(Option<PathBuf>),
    CancelButtonPressed,
    AddButtonPressed,
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Submit { name: String, path: Option<PathBuf> },
    Cancel,
}

#[derive(Debug, Clone)]
pub enum PickPathKind {
    Archive,
    Directory,
}

#[derive(Debug, Clone)]
pub struct AddModDialog {
    name: String,
    path: Option<PathBuf>,
}

impl AddModDialog {
    pub fn new() -> Self {
        Self {
            name: "".into(),
            path: None,
        }
    }

    fn clear(&mut self) {
        self.name.clear();
        self.path = None;
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NameChanged(name) => {
                self.name = name;
                Action::None
            }
            Message::PathChanged(path) => {
                self.path = Some(PathBuf::from(path));
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
                    .map(|f| f.path().to_path_buf())
                },
                Message::PathPicked,
            )),
            Message::PathPicked(path) => {
                self.path = path;
                Action::None
            }
            Message::CancelButtonPressed => {
                self.clear();

                Action::Cancel
            }
            Message::AddButtonPressed => {
                let name = self.name.clone();
                let path = self.path.clone();

                self.clear();

                Action::Submit { name, path }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let path_str = &self
            .path
            .as_ref()
            .map_or_else(String::new, |path| path.display().to_string());

        container(column![
            row![
                text(t!("name")),
                text_input("...", &self.name).on_input(Message::NameChanged)
            ],
            row![
                text(t!("path")),
                text_input("...", path_str).on_input(Message::PathChanged),
                button(Icon::Archive).on_press(Message::PickPath(PickPathKind::Archive)),
                button(Icon::Directory).on_press(Message::PickPath(PickPathKind::Directory))
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
        !self.name.is_empty()
    }
}

impl Default for AddModDialog {
    fn default() -> Self {
        Self::new()
    }
}
