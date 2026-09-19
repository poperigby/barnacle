use barnacle_gui::{
    icons::Icon,
    model::{Model, Mutation, ProfileRecord},
    persistence::state::UiStateStore,
    widgets::modal,
};
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{button, column, combo_box, row, rule, space, text},
};

use crate::ui::{add_mod_menu::AddModDialog, library_manager::LibraryManager, mod_list::ModList};

pub mod add_mod_menu;
pub mod library_manager;
pub mod mod_list;

#[derive(Debug, Clone)]
pub enum Message {
    LaunchButtonPressed,
    AddModButtonPressed,
    TabSelected(SelectedTab),
    LibraryManagerButtonPressed,
    ProfileSelected(ProfileRecord),

    // Children
    AddModDialog(add_mod_menu::Message),
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Launch,
}

#[derive(Debug, Clone, Default)]
pub enum SelectedTab {
    #[default]
    Mods,
    ItemGroup(String),
}

#[derive(Debug, Clone)]
pub struct Ui {
    show_library_manager: bool,
    show_add_mod_dialog: bool,

    selected_tab: SelectedTab,

    profile_selector_state: Option<combo_box::State<ProfileRecord>>,

    add_mod_dialog: AddModDialog,
    mod_list: ModList,
    library_manager: LibraryManager,
}

impl Ui {
    pub fn init(model: &Model, ui_state: &UiStateStore) -> Self {
        let mut ui = Self {
            show_library_manager: false,
            show_add_mod_dialog: false,

            selected_tab: SelectedTab::default(),

            profile_selector_state: None,

            // TODO: Move to Mods tab
            add_mod_dialog: AddModDialog::new(),
            mod_list: ModList::new(ui_state.clone()),
            library_manager: LibraryManager::new(),
        };

        ui.sync(model);

        ui
    }

    /// Synchronize runtime state with the given [`Model`]
    pub fn sync(&mut self, model: &Model) {
        self.profile_selector_state = match model.active_game().as_ref().map(|g| g.profiles.clone())
        {
            Some(profiles) if !profiles.is_empty() => Some(combo_box::State::new(profiles)),
            Some(_) | None => None,
        };

        self.library_manager.sync(model);
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TabSelected(tab) => {
                self.selected_tab = tab;

                Action::None
            }
            Message::LaunchButtonPressed => Action::Launch,
            Message::AddModButtonPressed => {
                self.show_add_mod_dialog = true;

                Action::None
            }
            Message::LibraryManagerButtonPressed => {
                self.show_library_manager = true;

                Action::None
            }
            Message::ProfileSelected(record) => {
                Action::Mutate(Mutation::ActivateProfile(record.object()))
            }

            // Children
            Message::AddModDialog(message) => match self.add_mod_dialog.update(message) {
                add_mod_menu::Action::None => Action::None,
                add_mod_menu::Action::Run(task) => Action::Run(task.map(Message::AddModDialog)),
                add_mod_menu::Action::Submit { name, path } => {
                    self.show_add_mod_dialog = false;

                    match path {
                        Some(path) if path.is_dir() => {
                            Action::Mutate(Mutation::AddModFromDir { name, path })
                        }
                        Some(path) => Action::Mutate(Mutation::AddModFromArchive { name, path }),
                        None => Action::Mutate(Mutation::AddEmptyMod { name }),
                    }
                }
                add_mod_menu::Action::Cancel => {
                    self.show_add_mod_dialog = false;

                    Action::None
                }
            },
            Message::ModList(message) => match self.mod_list.update(message) {
                mod_list::Action::None => Action::None,
                mod_list::Action::Run(task) => Action::Run(task.map(Message::ModList)),
                mod_list::Action::Mutate(mutation) => Action::Mutate(mutation),
            },
            Message::LibraryManager(message) => match self.library_manager.update(message) {
                library_manager::Action::None => Action::None,
                library_manager::Action::Run(task) => {
                    Action::Run(task.map(Message::LibraryManager))
                }
                library_manager::Action::Mutate(mutation) => Action::Mutate(mutation),
                library_manager::Action::Close => {
                    self.show_library_manager = false;

                    Action::None
                }
            },
        }
    }

    pub fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Message> {
        let profile_selector: Element<'a, Message> = match &self.profile_selector_state {
            Some(state) => row![
                text(t!("profile", { "count" => 1 })),
                combo_box(
                    state,
                    "...",
                    model.active_profile().as_ref(),
                    Message::ProfileSelected,
                )
            ]
            .into(),
            None => space::horizontal().into(),
        };

        let top_bar = row![
            button(text(t!("main_top-bar_launch"))).on_press(Message::LaunchButtonPressed),
            button(Icon::Wrench),
            profile_selector,
            space::horizontal(),
            button(Icon::Library).on_press(Message::LibraryManagerButtonPressed),
            button(Icon::Settings),
            button(Icon::Notifications)
        ];

        let mut tab_bar = row![button("Mods").on_press(Message::TabSelected(SelectedTab::Mods))];
        for group in model.item_entry_groups() {
            tab_bar = tab_bar.push(button(text(group.name.clone())).on_press(
                Message::TabSelected(SelectedTab::ItemGroup(group.name.clone())),
            ));
        }

        let action_bar = row![
            space::horizontal(),
            button(row!["Mod", Icon::Plus]).on_press_maybe(
                model
                    .active_profile()
                    .is_some()
                    .then_some(Message::AddModButtonPressed)
            )
        ];

        let main_pane: Element<'_, Message> = match &self.selected_tab {
            SelectedTab::Mods => column![self.mod_list.view(model).map(Message::ModList)].into(),
            SelectedTab::ItemGroup(group) => text!("{}", group).into(),
        };

        let content = column![
            top_bar,
            rule::horizontal(1),
            tab_bar,
            rule::horizontal(1),
            action_bar,
            main_pane
        ]
        .height(Length::Fill);

        if self.show_library_manager {
            modal(
                content,
                self.library_manager
                    .view(model)
                    .map(Message::LibraryManager),
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
