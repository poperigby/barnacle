use std::{path::PathBuf, sync::Arc};

use barnacle_lib::{Repository, repository::Profile};
use derive_more::{Deref, Display};
use fluent_i18n::t;
use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{button, column, combo_box, row, space, text},
};
use iced_aw::Spinner;
use parking_lot::RwLock;

use crate::{
    components::{
        add_mod_dialog::AddModDialog, library_manager::LibraryManager, mod_list::ModList,
    },
    config::GuiConfig,
    icons::icon,
    modal,
};

pub mod add_mod_dialog;
pub mod library_manager;
pub mod mod_list;

#[derive(Debug, Clone)]
pub enum Message {
    AppLoaded(AppData),
    AppLoadFailed(String),
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
    // Child components
    AddModDialog(add_mod_dialog::Message),
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
}

#[derive(Debug, Clone)]
enum State {
    Loading,
    Error(String),
    Ready {
        data: AppData,
        components: Components,
    },
}

#[derive(Debug, Clone)]
struct Components {
    profile_selector: ProfileSelector,
    show_library_manager: bool,
    show_add_mod_dialog: bool,

    add_mod_dialog: AddModDialog,
    mod_list: ModList,
    library_manager: LibraryManager,
}

impl Components {
    fn new(repo: &Repository, cfg: Arc<RwLock<GuiConfig>>) -> (Self, Task<Message>) {
        let (add_mod_dialog, add_mod_dialog_task) = AddModDialog::new(repo.clone());
        let mod_list = ModList::new(repo.clone(), cfg.clone());
        let (library_manager, library_manager_task) = LibraryManager::new(repo.clone());

        (
            Self {
                show_library_manager: false,
                show_add_mod_dialog: false,

                profile_selector: ProfileSelector {
                    state: combo_box::State::new(Vec::new()),
                    selected: None,
                },

                add_mod_dialog,
                mod_list,
                library_manager,
            },
            Task::batch([
                add_mod_dialog_task.map(Message::AddModDialog),
                library_manager_task.map(Message::LibraryManager),
            ]),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AppData {
    repo: Repository,
    active_profile: Option<ProfileOption>,
    profile_options: Vec<ProfileOption>,
}

impl AppData {
    async fn load(repo: Repository) -> Self {
        let mut active_profile = None;
        let mut profile_options = Vec::new();

        if let Some(active_game) = repo.active_game().await.unwrap() {
            for profile in active_game.profiles().await.unwrap() {
                let profile_option = ProfileOption {
                    entity: profile.clone(),
                    name: profile.name().await.unwrap().clone(),
                };

                if profile.is_active().await.unwrap() {
                    active_profile = Some(profile_option.clone())
                };

                profile_options.push(profile_option);
            }
        };

        AppData {
            repo,
            active_profile,
            profile_options,
        }
    }
}

pub struct App {
    state: State,
    cfg: Arc<RwLock<GuiConfig>>,

    title: String,
    theme: Theme,
}

impl App {
    pub const TITLE: &str = "Barnacle";

    pub fn new() -> (Self, Task<Message>) {
        let state = State::Loading;
        let cfg = Arc::new(RwLock::new(GuiConfig::load()));
        let theme = cfg.read().theme();

        (
            Self {
                state: state.clone(),
                cfg: cfg.clone(),

                title: Self::TITLE.to_string(),
                theme,
            },
            Self::load(),
        )
    }

    pub fn load() -> Task<Message> {
        Task::perform(
            {
                async {
                    let repo = Repository::new().await;
                    AppData::load(repo).await
                }
            },
            Message::AppLoaded,
        )
    }

    fn refresh(repo: &Repository) -> Task<Message> {
        Task::perform(AppData::load(repo.clone()), Message::AppLoaded)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppLoaded(data) => {
                let (mut components, components_task) =
                    Components::new(&data.repo, self.cfg.clone());

                components.profile_selector = ProfileSelector {
                    state: combo_box::State::new(data.profile_options.clone()),
                    selected: data.active_profile.clone(),
                };

                let mod_list_task = data
                    .active_profile
                    .as_ref()
                    .map(|profile| components.mod_list.refresh(profile).map(Message::ModList))
                    .unwrap_or_else(Task::none);

                self.state = State::Ready { data, components };

                Task::batch([components_task, mod_list_task])
            }
            Message::AppLoadFailed(e) => {
                self.state = State::Error(e);
                Task::none()
            }
            message => {
                let State::Ready { data, components } = &mut self.state else {
                    return Task::none();
                };

                let repo = data.repo.clone();

                match message {
                    Message::AddModDialog(message) => {
                        match components.add_mod_dialog.update(message) {
                            add_mod_dialog::Action::None => Task::none(),
                            add_mod_dialog::Action::Run(task) => task.map(Message::AddModDialog),
                            add_mod_dialog::Action::AddMod { name, path } => {
                                components.show_add_mod_dialog = false;
                                let repo = repo.clone();
                                Task::perform(
                                    async move {
                                        if let Some(active_game) = repo.active_game().await.unwrap()
                                        {
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
                                )
                            }
                            add_mod_dialog::Action::Cancel => {
                                components.show_add_mod_dialog = false;
                                Task::none()
                            }
                        }
                    }
                    Message::ModList(message) => match components.mod_list.update(message) {
                        mod_list::Action::None => Task::none(),
                        mod_list::Action::Run(task) => task.map(Message::ModList),
                    },
                    Message::LibraryManager(message) => {
                        match components.library_manager.update(message) {
                            library_manager::Action::None => Task::none(),
                            library_manager::Action::Run(task) => task.map(Message::LibraryManager),
                            library_manager::Action::CreateGame(new_game) => Task::perform(
                                async move {
                                    let repo = repo.clone();
                                    repo.add_game(&new_game.name, new_game.deploy_kind)
                                        .await
                                        .unwrap();
                                },
                                |_| Message::GameAdded,
                            ),
                            library_manager::Action::DeleteGame(game) => {
                                Task::perform(async move { game.remove().await.unwrap() }, |_| {
                                    Message::GameDeleted
                                })
                            }
                            library_manager::Action::ActivateGame(game) => {
                                Task::perform(async move { game.activate().await.unwrap() }, |_| {
                                    Message::GameActivated
                                })
                            }
                            library_manager::Action::CreateProfile { game, new_profile } => {
                                Task::perform(
                                    {
                                        let game = game.clone();
                                        async move { game.add_profile(&new_profile.name).await.unwrap() }
                                    },
                                    |_| Message::ProfileAdded,
                                )
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
                            library_manager::Action::DeleteProfile(profile) => Task::perform(
                                async {
                                    profile.remove().await.unwrap();
                                },
                                |_| Message::ProfileDeleted,
                            ),
                            library_manager::Action::Close => {
                                components.show_library_manager = false;
                                Task::none()
                            }
                        }
                    }
                    Message::AddModButtonPressed => {
                        components.show_add_mod_dialog = true;
                        Task::none()
                    }
                    Message::LibraryManagerButtonPressed => {
                        components.show_library_manager = true;
                        Task::none()
                    }
                    Message::ModAdded => {
                        if let Some(active_profile) = &components.profile_selector.selected {
                            components
                                .mod_list
                                .refresh(active_profile)
                                .map(Message::ModList)
                        } else {
                            Task::none()
                        }
                    }
                    Message::ProfileSelected(profile) => {
                        components.profile_selector.selected = Some(profile.clone());
                        Task::perform(
                            async {
                                profile.activate().await.unwrap();
                                profile.entity
                            },
                            Message::ProfileActivated,
                        )
                    }
                    // TODO: Update the mod list too. If the profile it's referring to is deleted, it needs
                    // to know.
                    Message::ProfileAdded | Message::ProfileDeleted => Task::batch([
                        Self::refresh(&repo),
                        components
                            .library_manager
                            .refresh()
                            .map(Message::LibraryManager),
                    ]),
                    Message::ProfileActivated(profile) => Task::batch([
                        Self::refresh(&repo),
                        components.mod_list.refresh(&profile).map(Message::ModList),
                    ]),
                    Message::GameAdded | Message::GameEdited | Message::GameDeleted => components
                        .library_manager
                        .refresh()
                        .map(Message::LibraryManager),
                    Message::GameActivated => Task::batch([
                        components
                            .library_manager
                            .refresh()
                            .map(Message::LibraryManager),
                        Self::refresh(&repo),
                    ]),
                    // TODO: Already handled in the outer match, but this is gross. Need to
                    // restructure this.
                    Message::AppLoaded(_) | Message::AppLoadFailed(_) => Task::none(),
                }
            }
        }
    }

    // Render the application and pass along messages from components to update()
    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => Spinner::new().into(),
            State::Error(_) => panic!("ERROR"),
            State::Ready { components, .. } => {
                let content = column![
                    // Top bar
                    row![
                        button(text(t!("main_top-bar_launch-game", { "count" => 1 }))),
                        button(icon("wrench")),
                        text(t!("profile", { "count" => 1 })),
                        combo_box(
                            &components.profile_selector.state,
                            "...",
                            components.profile_selector.selected.as_ref(),
                            Message::ProfileSelected
                        ),
                        space::horizontal(),
                        button(icon("library")).on_press(Message::LibraryManagerButtonPressed),
                        button(icon("settings")),
                        button(icon("notifications"))
                    ],
                    // Action bar
                    row![
                        button(text(t!("main_action-bar_add-mod", { "count" => 1 })))
                            .on_press_maybe(
                                components
                                    .profile_selector
                                    .selected
                                    .is_some()
                                    .then_some(Message::AddModButtonPressed)
                            )
                    ],
                    // Mod list
                    components.mod_list.view().map(Message::ModList),
                ]
                .height(Fill);

                if components.show_library_manager {
                    modal(
                        content,
                        components
                            .library_manager
                            .view()
                            .map(Message::LibraryManager),
                        None,
                    )
                } else if components.show_add_mod_dialog {
                    modal(
                        content,
                        components.add_mod_dialog.view().map(Message::AddModDialog),
                        None,
                    )
                } else {
                    content.into()
                }
            }
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

#[derive(Debug, Clone)]
struct ProfileSelector {
    state: combo_box::State<ProfileOption>,
    selected: Option<ProfileOption>,
}

#[derive(Clone, Debug, Display, Deref)]
#[display("{}", name)]
pub struct ProfileOption {
    #[deref]
    entity: Profile,
    name: String,
}
