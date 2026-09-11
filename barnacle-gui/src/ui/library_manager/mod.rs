use barnacle_gui::{
    icons::Icon,
    modal,
    model::{GameItem, Model, Mutation},
};
use barnacle_lib::repository::Game;
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, rule, scrollable, space, text},
};

use crate::ui::library_manager::menus::{edit_profile, new_game, new_profile};

pub mod game_details;
pub mod menus;

#[derive(Debug, Clone)]
pub enum Message {
    CloseButtonPressed,
    NewGameButtonPressed,
    ActivateButtonPressed(Game),
    GameRowSelected(GameItem),

    GameDetails(game_details::Message),
    NewGameMenu(new_game::Message),
    NewProfileMenu(new_profile::Message),
    EditProfileMenu(edit_profile::Message),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Close,
}

#[derive(Debug, Clone)]
pub enum MenuVisibility {
    NewGame,
    NewProfile,
    EditProfile,
    None,
}

#[derive(Debug, Clone)]
pub struct LibraryManager {
    selected_game: Option<GameItem>,

    menu_visiblity: MenuVisibility,

    game_details: game_details::GameDetails,

    new_game_menu: new_game::Menu,
    new_profile_menu: new_profile::Menu,
    edit_profile_menu: edit_profile::Menu,
}

impl LibraryManager {
    pub fn new() -> Self {
        let game_details = game_details::GameDetails::new();

        let new_game_menu = new_game::Menu::new();
        let new_profile_menu = new_profile::Menu::new();
        let edit_profile_menu = edit_profile::Menu::new();

        Self {
            selected_game: None,

            menu_visiblity: MenuVisibility::None,

            game_details,

            new_game_menu,
            new_profile_menu,
            edit_profile_menu,
        }
    }

    pub fn sync(&mut self, model: &Model) {
        // Select the active game if there isn't one already selected. If there is, we can refresh
        // the data from the model.
        self.selected_game = self
            .selected_game
            .as_ref()
            .and_then(|item| model.game(&item.handle()))
            .or_else(|| model.active_game().clone());
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::CloseButtonPressed => Action::Close,
            Message::NewGameButtonPressed => {
                self.menu_visiblity = MenuVisibility::NewGame;

                Action::None
            }
            Message::ActivateButtonPressed(game) => Action::Mutate(Mutation::ActivateGame(game)),
            Message::GameRowSelected(game) => {
                self.selected_game = Some(game.clone());

                Action::None
            }
            Message::GameDetails(message) => match self.game_details.update(message) {
                game_details::Action::None => Action::None,
                game_details::Action::Run(task) => Action::Run(task.map(Message::GameDetails)),
                game_details::Action::NewProfile => {
                    self.menu_visiblity = MenuVisibility::NewProfile;

                    Action::None
                }
                game_details::Action::EditProfile {
                    profile,
                    initial_name,
                } => {
                    self.edit_profile_menu.load(profile, initial_name);
                    self.menu_visiblity = MenuVisibility::EditProfile;

                    Action::None
                }
                game_details::Action::Mutate(mutation) => Action::Mutate(mutation),
            },
            Message::NewGameMenu(message) => match self.new_game_menu.update(message) {
                new_game::Action::None => Action::None,
                new_game::Action::Run(task) => Action::Run(task.map(Message::NewGameMenu)),
                new_game::Action::Mutate(mutation) => match mutation {
                    Mutation::CreateGame { .. } => {
                        self.menu_visiblity = MenuVisibility::None;

                        Action::Mutate(mutation)
                    }
                    _ => Action::Mutate(mutation),
                },
                new_game::Action::Cancel => {
                    self.menu_visiblity = MenuVisibility::None;
                    Action::None
                }
            },
            Message::NewProfileMenu(message) => match self.new_profile_menu.update(message) {
                new_profile::Action::None => Action::None,
                new_profile::Action::Run(task) => Action::Run(task.map(Message::NewProfileMenu)),
                new_profile::Action::Create { name } => {
                    self.menu_visiblity = MenuVisibility::None;

                    if let Some(game) = &self.selected_game {
                        Action::Mutate(Mutation::CreateProfile {
                            game: game.handle(),
                            name,
                        })
                    } else {
                        Action::None
                    }
                }
                new_profile::Action::Cancel => {
                    self.menu_visiblity = MenuVisibility::None;

                    Action::None
                }
            },
            Message::EditProfileMenu(message) => match self.edit_profile_menu.update(message) {
                edit_profile::Action::None => Action::None,
                edit_profile::Action::Run(task) => Action::Run(task.map(Message::EditProfileMenu)),
                edit_profile::Action::Mutate(mutation) => Action::Mutate(mutation),
                edit_profile::Action::Cancel => Action::None,
            },
        }
    }

    pub fn view(&self, model: &Model) -> Element<'_, Message> {
        let title_bar = row![
            text(t!("library-manager_title")),
            space::horizontal(),
            button(Icon::Close).on_press(Message::CloseButtonPressed)
        ];

        let new_game_button = button(row![
            Icon::Plus,
            text(t!("library-manager_new-game", { "count" => 1 }))
        ])
        .on_press(Message::NewGameButtonPressed);

        let games_list = column![
            text(t!("game", { "count" => 2 })),
            rule::horizontal(1),
            scrollable(Column::with_children(
                model.games().iter().map(|g| self.game_row(g))
            )),
            space::vertical(),
            new_game_button
        ];

        let content = column![
            title_bar,
            row![
                games_list.width(Length::FillPortion(1)),
                self.game_details
                    .view(model, &self.selected_game)
                    .map(Message::GameDetails)
            ]
            .padding(20)
        ]
        .into();

        let modal_content: Option<Element<'_, Message>> = match self.menu_visiblity {
            MenuVisibility::NewGame => Some(self.new_game_menu.view().map(Message::NewGameMenu)),
            MenuVisibility::NewProfile => {
                Some(self.new_profile_menu.view().map(Message::NewProfileMenu))
            }
            MenuVisibility::EditProfile => {
                Some(self.edit_profile_menu.view().map(Message::EditProfileMenu))
            }
            MenuVisibility::None => None,
        };

        container(if let Some(modal_content) = modal_content {
            modal(content, modal_content, None)
        } else {
            content
        })
        .width(800)
        .height(600)
        .style(container::rounded_box)
        .into()
    }

    pub fn game_row<'a>(&self, item: &GameItem) -> Element<'a, Message> {
        let mut content = row![text(item.name.clone()), space::horizontal()];

        if item.active {
            content = content.push(Icon::Check);
        }

        let style = if Some(item) == self.selected_game.as_ref() {
            button::primary
        } else {
            button::subtle
        };

        button(content)
            .width(Length::Fill)
            .style(style)
            .on_press(Message::GameRowSelected(item.clone()))
            .into()
    }
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self::new()
    }
}
