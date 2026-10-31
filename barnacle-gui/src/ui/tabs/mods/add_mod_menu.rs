use std::{env, path::PathBuf};

use anyhow::Context;
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
    PickPath(PickPathKind),
    PathPicked(Option<PathBuf>),
    PathChanged(String),

    ImportProgress(FileProgress),
    ImportDone,

    NameChanged(String),

    CancelButtonPressed,
    AddButtonPressed,
}

pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation<Message>),
    Close,
}

#[derive(Debug, Clone, Default)]
pub enum Page {
    #[default]
    ChooseSource,
    Preparing,
    Importing {
        progress: FileProgress,
    },
    Configure,
}

#[derive(Debug, Clone)]
pub enum PickPathKind {
    Archive,
    Directory,
}

#[derive(Debug, Clone)]
pub struct AddModDialog {
    page: Page,
    name: String,
    path: Option<PathBuf>,
}

impl AddModDialog {
    pub fn new() -> Self {
        Self {
            page: Page::default(),
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
            Message::ImportProgress(progress) => {
                self.page = Page::Importing { progress };

                Action::None
            }
            Message::ImportDone => {
                self.page = Page::Configure;

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

                Action::Close
            }
            Message::AddButtonPressed => {
                let name = self.name.clone();
                let path = self.path.clone();

                match path {
                    Some(path) if path.is_dir() => {
                        self.page = Page::Preparing;

                        Action::Mutate(Mutation::sip(
                            move |repo, progress| async move {
                                let game = repo.active_game().await?.context("no active game")?;

                                game.new_mod(&name).from_dir(&path, Some(progress)).await;

                                Ok(())
                            },
                            Message::ImportProgress,
                            || Message::ImportDone,
                        ))
                    }
                    Some(path) => {
                        self.page = Page::Preparing;

                        Action::Mutate(Mutation::sip(
                            move |repo, progress| async move {
                                let game = repo.active_game().await?.context("no active game")?;

                                game.new_mod(&name)
                                    .from_archive(&path)
                                    .install(Some(progress))
                                    .await;

                                Ok(())
                            },
                            Message::ImportProgress,
                            || Message::ImportDone,
                        ))
                    }
                    None => {
                        self.page = Page::Preparing;

                        Action::Mutate(Mutation::add_empty_mod(name))
                    }
                }
            }
        }
    }

    pub fn view(&self, translations: &Translations) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.page {
            Page::ChooseSource => {
                let path_str = &self
                    .path
                    .as_ref()
                    .map_or_else(String::new, |path| path.display().to_string());

                column![
                    row![
                        translations.path().element(),
                        text_input("...", path_str).on_input(Message::PathChanged),
                        button(Icon::Archive).on_press(Message::PickPath(PickPathKind::Archive)),
                        button(Icon::Directory)
                            .on_press(Message::PickPath(PickPathKind::Directory))
                    ],
                    space::vertical(),
                    row![
                        space::horizontal(),
                        button(translations.cancel().element())
                            .on_press(Message::CancelButtonPressed),
                        button(translations.add().element())
                            .on_press_maybe(self.validate().then_some(Message::AddButtonPressed))
                    ]
                ]
                .into()
            }
            Page::Preparing => "Preparing".into(),
            Page::Importing { progress } => {
                let range = 0.0..=(progress.total() as f32);
                let current = progress.current() as f32;

                progress_bar(range, current).into()
            }
            Page::Configure => column![row![
                translations.name().element(),
                text_input("...", &self.name).on_input(Message::NameChanged)
            ]]
            .into(),
        };

        container(content)
            .padding(20)
            .width(400)
            .height(400)
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
