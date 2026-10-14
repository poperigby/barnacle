use barnacle_gui::{
    TranslationElementExt,
    icons::Icon,
    model::{Model, Mutation, ProfileRecord},
    persistence::state::UiStateStore,
    widgets::modal,
};
use barnacle_l10n::Translations;
use barnacle_lib::repository::items;
use iced::{
    Element, Length, Task,
    widget::{Button, button, column, combo_box, row, rule, space, text},
};

use crate::ui::{
    library_manager::LibraryManager,
    tabs::{
        item_group::ItemGroupTab,
        mods::{ModsTab, add_mod_menu::AddModDialog},
    },
};

pub mod library_manager;
pub mod tabs;

#[derive(Debug, Clone)]
pub enum Message {
    LaunchButtonPressed,
    AddModButtonPressed,
    TabSelected(Tab),
    LibraryManagerButtonPressed,
    ProfileSelected(ProfileRecord),

    // Children
    AddModDialog(tabs::mods::add_mod_menu::Message),
    ModList(tabs::mods::Message),
    ItemGroup(tabs::item_group::Message),
    LibraryManager(library_manager::Message),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Launch,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Tab {
    #[default]
    Mods,
    ItemGroup(items::Kind),
}

#[derive(Debug, Clone)]
pub struct Ui {
    show_library_manager: bool,
    show_add_mod_dialog: bool,

    selected_tab: Tab,

    profile_selector_state: Option<combo_box::State<ProfileRecord>>,

    add_mod_dialog: AddModDialog,
    mods_tab: ModsTab,
    library_manager: LibraryManager,
}

impl Ui {
    pub fn init(model: &Model, ui_state: &UiStateStore) -> Self {
        let mut ui = Self {
            show_library_manager: false,
            show_add_mod_dialog: false,

            selected_tab: Tab::default(),

            profile_selector_state: None,

            // TODO: Move to Mods tab
            add_mod_dialog: AddModDialog::new(),
            mods_tab: ModsTab::new(ui_state.clone()),
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
                tabs::mods::add_mod_menu::Action::None => Action::None,
                tabs::mods::add_mod_menu::Action::Run(task) => {
                    Action::Run(task.map(Message::AddModDialog))
                }
                tabs::mods::add_mod_menu::Action::Submit { name, path } => {
                    self.show_add_mod_dialog = false;

                    match path {
                        Some(path) if path.is_dir() => {
                            Action::Mutate(Mutation::AddModFromDir { name, path })
                        }
                        Some(path) => Action::Mutate(Mutation::AddModFromArchive { name, path }),
                        None => Action::Mutate(Mutation::AddEmptyMod { name }),
                    }
                }
                tabs::mods::add_mod_menu::Action::Cancel => {
                    self.show_add_mod_dialog = false;

                    Action::None
                }
            },
            Message::ModList(message) => match self.mods_tab.update(message) {
                tabs::mods::Action::None => Action::None,
                tabs::mods::Action::Run(task) => Action::Run(task.map(Message::ModList)),
                tabs::mods::Action::Mutate(mutation) => Action::Mutate(mutation),
            },
            Message::ItemGroup(message) => match ItemGroupTab::update(message) {
                tabs::item_group::Action::None => Action::None,
                tabs::item_group::Action::Run(task) => Action::Run(task.map(Message::ItemGroup)),
                tabs::item_group::Action::Mutate(mutation) => Action::Mutate(mutation),
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

    pub fn view<'a>(
        &'a self,
        model: &'a Model,
        translations: &'a Translations,
    ) -> Element<'a, Message> {
        let profile_selector: Element<'a, Message> = match &self.profile_selector_state {
            Some(state) => row![
                translations.main_top_bar_add_mod().element(),
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
            button(translations.main_top_bar_launch().element())
                .on_press(Message::LaunchButtonPressed),
            button(Icon::Wrench),
            profile_selector,
            space::horizontal(),
            button(Icon::Library).on_press(Message::LibraryManagerButtonPressed),
            button(Icon::Settings),
            button(Icon::Notifications)
        ];

        let mut tab_bar =
            row![self.tab_button(translations, Tab::Mods, Message::TabSelected(Tab::Mods))];

        for group in model.item_groups() {
            tab_bar = tab_bar.push(self.tab_button(
                translations,
                Tab::ItemGroup(group.kind),
                Message::TabSelected(Tab::ItemGroup(group.kind)),
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
            Tab::Mods => column![self.mods_tab.view(model).map(Message::ModList)].into(),
            Tab::ItemGroup(kind) => {
                let group = model
                    .item_groups()
                    .iter()
                    .find(|g| g.kind == *kind)
                    .unwrap();
                ItemGroupTab::view(group, translations).map(Message::ItemGroup)
            }
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
                    .view(model, translations)
                    .map(Message::LibraryManager),
                None,
            )
        } else if self.show_add_mod_dialog {
            modal(
                content,
                self.add_mod_dialog
                    .view(translations)
                    .map(Message::AddModDialog),
                None,
            )
        } else {
            content.into()
        }
    }

    fn tab_button(
        &self,
        translations: &Translations,
        tab: Tab,
        message: Message,
    ) -> Button<'_, Message> {
        let style = if self.selected_tab == tab {
            button::primary
        } else {
            button::subtle
        };

        let label = text(match tab {
            Tab::Mods => translations.object_mod("many").to_string(),
            Tab::ItemGroup(kind) => kind.name(translations),
        });

        button(label).style(style).on_press(message)
    }
}
