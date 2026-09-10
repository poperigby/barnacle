use barnacle_gui::modal;
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{button, column, combo_box, row, space, text},
};

use crate::{
    icons::Icon,
    model::{Model, Mutation, ProfileItem},
    persistence::state::UiStateStore,
    ui::{add_mod_dialog::AddModDialog, library_manager::LibraryManager, mod_list::ModList},
};

pub mod add_mod_dialog;
pub mod library_manager;
pub mod mod_list;

#[derive(Debug, Clone)]
pub enum Message {
    AddModButtonPressed,
    LibraryManagerButtonPressed,
    ProfileSelected(ProfileItem),

    // Children
    AddModDialog(add_mod_dialog::Message),
    ModList(mod_list::Message),
    LibraryManager(library_manager::Message),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct Ui {
    show_library_manager: bool,
    show_add_mod_dialog: bool,

    profile_selector_state: Option<combo_box::State<ProfileItem>>,

    add_mod_dialog: AddModDialog,
    mod_list: ModList,
    library_manager: LibraryManager,
}

impl Ui {
    pub fn init(model: &Model, ui_state: &UiStateStore) -> Self {
        let mut ui = Self {
            show_library_manager: false,
            show_add_mod_dialog: false,

            profile_selector_state: None,

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
            Message::AddModButtonPressed => {
                self.show_add_mod_dialog = true;

                Action::None
            }
            Message::LibraryManagerButtonPressed => {
                self.show_library_manager = true;

                Action::None
            }
            Message::ProfileSelected(row) => {
                Action::Mutate(Mutation::ActivateProfile(row.handle()))
            }

            // Children
            Message::AddModDialog(message) => match self.add_mod_dialog.update(message) {
                add_mod_dialog::Action::None => Action::None,
                add_mod_dialog::Action::Run(task) => Action::Run(task.map(Message::AddModDialog)),
                add_mod_dialog::Action::Submit { name, path } => {
                    self.show_add_mod_dialog = false;

                    Action::Mutate(Mutation::AddMod { name, path })
                }
                add_mod_dialog::Action::Cancel => {
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
            Some(state) => combo_box(
                &state,
                "...",
                model.active_profile().as_ref(),
                Message::ProfileSelected,
            )
            .into(),
            None => space::horizontal().into(),
        };

        let top_bar = row![
            button(text(t!("main_top-bar_launch-game", { "count" => 1 }))),
            button(Icon::Wrench),
            text(t!("profile", { "count" => 1 })),
            profile_selector,
            space::horizontal(),
            button(Icon::Library).on_press(Message::LibraryManagerButtonPressed),
            button(Icon::Settings),
            button(Icon::Notifications)
        ];

        let action_bar = row![
            button(text(t!("main_action-bar_add-mod", { "count" => 1 }))).on_press_maybe(
                model
                    .active_profile()
                    .is_some()
                    .then_some(Message::AddModButtonPressed)
            )
        ];

        let main_pane: Element<'_, Message> =
            column![self.mod_list.view(model).map(Message::ModList)].into();

        let content = column![top_bar, action_bar, main_pane].height(Length::Fill);

        if self.show_library_manager {
            modal(
                content,
                self.library_manager
                    .view(&model)
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
