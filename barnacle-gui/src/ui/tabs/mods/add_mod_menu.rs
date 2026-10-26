use std::{env, path::PathBuf};

use barnacle_gui::{TranslationElementExt, icons::Icon, model::Mutation};
use barnacle_l10n::Translations;
use barnacle_lib::mods::builder::FileProgress;
use iced::{
    Element, Task,
    widget::{button, column, container, progress_bar, row, space, text_input},
};
use rfd::AsyncFileDialog;

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
    Mutate(Mutation),
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

                match path {
                    Some(path) if path.is_dir() => {
                        Action::Mutate(Mutation::AddModFromDir { name, path })
                    }
                    Some(path) => Action::Mutate(Mutation::AddModFromArchive { name, path }),
                    None => Action::Mutate(Mutation::AddEmptyMod { name }),
                }
            }
        }
    }

    pub fn view(
        &self,
        file_progress: &Option<FileProgress>,
        translations: &Translations,
    ) -> Element<'_, Message> {
        let path_str = &self
            .path
            .as_ref()
            .map_or_else(String::new, |path| path.display().to_string());

        let progress_bar: Element<'_, Message> = if let Some(progress) = file_progress {
            let range = 0.0..=(progress.total() as f32);
            let current = progress.current() as f32;

            progress_bar(range, current).into()
        } else {
            space().into()
        };

        container(column![
            row![
                translations.name().element(),
                text_input("...", &self.name).on_input(Message::NameChanged)
            ],
            row![
                translations.path().element(),
                text_input("...", path_str).on_input(Message::PathChanged),
                button(Icon::Archive).on_press(Message::PickPath(PickPathKind::Archive)),
                button(Icon::Directory).on_press(Message::PickPath(PickPathKind::Directory))
            ],
            space::vertical(),
            progress_bar,
            row![
                space::horizontal(),
                button(translations.cancel().element()).on_press(Message::CancelButtonPressed),
                button(translations.add().element())
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
