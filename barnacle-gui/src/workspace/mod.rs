use std::{path::PathBuf, sync::Arc};

use barnacle_lib::{Repository, repository::Profile};
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{button, column, combo_box, row, space, text},
};
use parking_lot::RwLock;

use crate::{
    AppData,
    config::GuiConfig,
    data::ProfileOption,
    icons::Icon,
    modal,
    workspace::{add_mod_dialog::AddModDialog, library_manager::LibraryManager, mod_list::ModList},
};

pub mod add_mod_dialog;
pub mod library_manager;
pub mod mod_list;

#[derive(Debug, Clone)]
pub enum Message {
    AddModButtonPressed,
    LibraryManagerButtonPressed,
    ModAdded,
    GameAdded,
    GameEdited,
    GameDeleted,
    GameActivated,
    ProfileAdded,
    ProfileDeleted,
    ProfileSelected(ProfileOption),
    ProfileActivated(Profile),

    // Children
    AddModDialog(add_mod_dialog::Message),
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Refresh,
}

/// The user facing working area that exists once the app is loaded
#[derive(Debug, Clone)]
pub struct Workspace {
    profile_selector: ProfileSelector,
    show_library_manager: bool,
    show_add_mod_dialog: bool,

    add_mod_dialog: AddModDialog,
    mod_list: Option<ModList>,
    library_manager: LibraryManager,
}

impl Workspace {
    pub fn init(
        repo: &Repository,
        data: &AppData,
        cfg: Arc<RwLock<GuiConfig>>,
    ) -> (Self, Task<Message>) {
        let (add_mod_dialog, add_mod_dialog_task) = AddModDialog::new(repo.clone());
        let (mod_list, mod_list_task) = data
            .active_profile
            .as_ref()
            .map(|active_profile| {
                let (mod_list, task) = ModList::new(cfg.clone(), active_profile.handle.clone());
                (Some(mod_list), task)
            })
            .unwrap_or_else(|| (None, Task::none()));
        let (library_manager, library_manager_task) = LibraryManager::new(repo.clone());

        (
            Self {
                show_library_manager: false,
                show_add_mod_dialog: false,

                profile_selector: ProfileSelector {
                    state: combo_box::State::new(data.profile_options.clone()),
                    selected: data.active_profile.clone(),
                },

                add_mod_dialog,
                mod_list,
                library_manager,
            },
            Task::batch([
                add_mod_dialog_task.map(Message::AddModDialog),
                mod_list_task.map(Message::ModList),
                library_manager_task.map(Message::LibraryManager),
            ]),
        )
    }

    /// Synchronize the workspace UI with the newly refreshed application data
    pub fn sync(&mut self, data: &AppData) -> Task<Message> {
        self.profile_selector = ProfileSelector {
            state: combo_box::State::new(data.profile_options.clone()),
            selected: data.active_profile.clone(),
        };

        let mod_list_task = match (&self.mod_list, &data.active_profile) {
            (Some(mod_list), Some(active_profile)) => mod_list
                .refresh(active_profile.handle.clone())
                .map(Message::ModList),
            _ => Task::none(),
        };

        // We want to let [`App`] know that it needs to refresh these children as well
        Task::batch([
            self.library_manager.refresh().map(Message::LibraryManager),
            mod_list_task,
        ])
    }

    pub fn update(&mut self, repo: &Repository, message: Message) -> Action {
        match message {
            Message::AddModDialog(message) => match self.add_mod_dialog.update(message) {
                add_mod_dialog::Action::None => Action::None,
                add_mod_dialog::Action::Run(task) => Action::Run(task.map(Message::AddModDialog)),
                add_mod_dialog::Action::AddMod { name, path } => {
                    self.show_add_mod_dialog = false;
                    let repo = repo.clone();
                    Action::Run(Task::perform(
                        async move {
                            if let Some(active_game) = repo.active_game().await.unwrap() {
                                let mod_ = active_game
                                    .add_mod(&name, Some(&PathBuf::from(path)))
                                    .await
                                    .unwrap();

                                if let Some(active_profile) =
                                    active_game.active_profile().await.unwrap()
                                {
                                    active_profile.add_mod_entry(mod_).await.unwrap();
                                }
                            }
                        },
                        |_| Message::ModAdded,
                    ))
                }
                add_mod_dialog::Action::Cancel => {
                    self.show_add_mod_dialog = false;
                    Action::None
                }
            },
            Message::ModList(message) => {
                let Some(mod_list) = &mut self.mod_list else {
                    return Action::None;
                };

                match mod_list.update(message) {
                    mod_list::Action::None => Action::None,
                    mod_list::Action::Run(task) => Action::Run(task.map(Message::ModList)),
                }
            }
            Message::LibraryManager(message) => {
                let repo = repo.clone();

                match self.library_manager.update(message) {
                    library_manager::Action::None => Action::None,
                    library_manager::Action::Run(task) => {
                        Action::Run(task.map(Message::LibraryManager))
                    }
                    library_manager::Action::CreateGame(new_game) => Action::Run(Task::perform(
                        async move {
                            repo.add_game(&new_game.name, new_game.deploy_kind)
                                .await
                                .unwrap();
                        },
                        |_| Message::GameAdded,
                    )),
                    library_manager::Action::DeleteGame(game) => Action::Run(Task::perform(
                        async move { game.remove().await.unwrap() },
                        |_| Message::GameDeleted,
                    )),
                    library_manager::Action::ActivateGame(game) => Action::Run(Task::perform(
                        async move { game.activate().await.unwrap() },
                        |_| Message::GameActivated,
                    )),
                    library_manager::Action::CreateProfile { game, new_profile } => {
                        Action::Run(Task::perform(
                            {
                                let game = game.clone();
                                async move { game.add_profile(&new_profile.name).await.unwrap() }
                            },
                            |_| Message::ProfileAdded,
                        ))
                    }
                    // library_manager::Action::EditGame(edit) => Task::perform(
                    //     async move {
                    //         spawn_blocking(move || {
                    //             edit.game.set_name(&edit.name).unwrap();
                    //             edit.game.set_deploy_kind(edit.deploy_kind).unwrap();
                    //         })
                    //         .await
                    //         .unwrap()
                    //     },
                    //     |_| ReadyMessage::GameEdited,
                    // ),
                    library_manager::Action::DeleteProfile(profile) => Action::Run(Task::perform(
                        async {
                            profile.remove().await.unwrap();
                        },
                        |_| Message::ProfileDeleted,
                    )),
                    library_manager::Action::Close => {
                        self.show_library_manager = false;
                        Action::None
                    }
                }
            }
            Message::AddModButtonPressed => {
                self.show_add_mod_dialog = true;
                Action::None
            }
            Message::LibraryManagerButtonPressed => {
                self.show_library_manager = true;
                Action::None
            }
            Message::ModAdded => {
                if let (Some(active_profile), Some(mod_list)) =
                    (&self.profile_selector.selected, &self.mod_list)
                {
                    Action::Run(
                        mod_list
                            .refresh(active_profile.handle.clone())
                            .map(Message::ModList),
                    )
                } else {
                    Action::None
                }
            }
            Message::ProfileSelected(profile) => {
                self.profile_selector.selected = Some(profile.clone());
                Action::Run(Task::perform(
                    async {
                        profile.handle.activate().await.unwrap();
                        profile.handle
                    },
                    Message::ProfileActivated,
                ))
            }
            // TODO: Update the mod list too. If the profile it's referring to is deleted, it needs
            // to know.
            Message::ProfileAdded | Message::ProfileDeleted => Action::Refresh,
            Message::ProfileActivated(_) => Action::Refresh,
            Message::GameAdded | Message::GameEdited | Message::GameDeleted => {
                Action::Run(self.library_manager.refresh().map(Message::LibraryManager))
            }
            Message::GameActivated => Action::Refresh,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let top_bar = row![
            button(text(t!("main_top-bar_launch-game", { "count" => 1 }))),
            button(Icon::Wrench),
            text(t!("profile", { "count" => 1 })),
            combo_box(
                &self.profile_selector.state,
                "...",
                self.profile_selector.selected.as_ref(),
                Message::ProfileSelected
            ),
            space::horizontal(),
            button(Icon::Library).on_press(Message::LibraryManagerButtonPressed),
            button(Icon::Settings),
            button(Icon::Notifications)
        ];

        let action_bar = row![
            button(text(t!("main_action-bar_add-mod", { "count" => 1 }))).on_press_maybe(
                self.profile_selector
                    .selected
                    .is_some()
                    .then_some(Message::AddModButtonPressed)
            )
        ];

        let main_pane: Element<'_, Message> = match &self.mod_list {
            Some(list) => list.view().map(Message::ModList),
            None => text("No active profile").into(),
        };

        let content = column![top_bar, action_bar, main_pane].height(Length::Fill);
        if self.show_library_manager {
            modal(
                content,
                self.library_manager.view().map(Message::LibraryManager),
                None,
            )
        } else if self.show_add_mod_dialog {
            modal(
                content,
                self.add_mod_dialog.view().map(Message::AddModDialog),
                None,
            )
        } else {
            content.into()
        }
    }
}

#[derive(Debug, Clone)]
struct ProfileSelector {
    state: combo_box::State<ProfileOption>,
    selected: Option<ProfileOption>,
}
