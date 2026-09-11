use crate::{
    icons::Icon,
    model::{GameItem, Model, Mutation},
};
use barnacle_gui::modal;
use barnacle_lib::repository::Game;
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, rule, scrollable, space, text},
};

pub mod game_details;
pub mod new_game_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    CloseButtonPressed,
    NewGameButtonPressed,
    ActivateButtonPressed(Game),
    GameRowSelected(GameItem),
    // Components
    NewGameDialog(new_game_dialog::Message),
    GameDetails(game_details::Message),
}

/// Action used for communicating with the parent component
#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Close,
}

#[derive(Debug, Clone)]
pub struct LibraryManager {
    selected_game: Option<GameItem>,
    show_new_game_dialog: bool,
    // Components
    new_game_dialog: new_game_dialog::Dialog,
    game_details: game_details::Tab,
}

impl LibraryManager {
    pub fn new() -> Self {
        let new_game_dialog = new_game_dialog::Dialog::new();
        let game_details = game_details::Tab::new();

        Self {
            selected_game: None,
            show_new_game_dialog: false,
            new_game_dialog,
            game_details,
        }
    }

    pub fn sync(&mut self, model: &Model) {
        self.selected_game = match &self.selected_game {
            Some(game) if model.games().contains(game) => Some(game.clone()),
            Some(_) | None => model.active_game().clone(),
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::CloseButtonPressed => Action::Close,
            Message::NewGameButtonPressed => {
                self.show_new_game_dialog = true;

                Action::None
            }
            Message::ActivateButtonPressed(game) => Action::Mutate(Mutation::ActivateGame(game)),
            Message::GameRowSelected(game) => {
                self.selected_game = Some(game.clone());

                Action::None
            }
            Message::NewGameDialog(message) => match self.new_game_dialog.update(message) {
                new_game_dialog::Action::None => Action::None,
                new_game_dialog::Action::Run(task) => Action::Run(task.map(Message::NewGameDialog)),
                new_game_dialog::Action::Mutate(mutation) => match mutation {
                    Mutation::CreateGame { .. } => {
                        self.show_new_game_dialog = false;

                        Action::Mutate(mutation)
                    }
                    _ => Action::Mutate(mutation),
                },
                new_game_dialog::Action::Cancel => {
                    self.show_new_game_dialog = false;
                    Action::None
                }
            },
            Message::GameDetails(message) => match self.game_details.update(message) {
                game_details::Action::None => Action::None,
                game_details::Action::Run(task) => Action::Run(task.map(Message::GameDetails)),
                game_details::Action::Create { name } => match &self.selected_game {
                    Some(game) => Action::Mutate(Mutation::CreateProfile {
                        game: game.handle().clone(),
                        name,
                    }),
                    None => Action::None,
                },
                game_details::Action::Mutate(mutation) => Action::Mutate(mutation),
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

        container(if self.show_new_game_dialog {
            modal(
                content,
                self.new_game_dialog.view().map(Message::NewGameDialog),
                None,
            )
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
