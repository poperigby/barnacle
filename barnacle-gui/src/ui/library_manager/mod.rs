use crate::{
    icons::Icon,
    model::{GameRow, Model, Mutation},
};
use barnacle_gui::modal;
use barnacle_lib::repository::Game;
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, rule, scrollable, space, text},
};

pub mod new_game_dialog;
pub mod profiles_tab;

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    CloseButtonPressed,
    NewGameButtonPressed,
    ActivateButtonPressed(Game),
    GameRowSelected(Game),
    // Components
    NewGameDialog(new_game_dialog::Message),
    ProfilesTab(profiles_tab::Message),
}

/// Action used for communicating with the parent component
#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Close,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum TabId {
    Overview,
    #[default]
    Profiles,
}

#[derive(Debug, Clone)]
pub struct LibraryManager {
    active_tab: TabId,
    selected_game: Option<Game>,
    show_new_game_dialog: bool,
    // Components
    new_game_dialog: new_game_dialog::Dialog,
    profiles_tab: profiles_tab::Tab,
}

impl LibraryManager {
    pub fn new() -> Self {
        let new_game_dialog = new_game_dialog::Dialog::new();
        let profiles_tab = profiles_tab::Tab::new();

        Self {
            active_tab: TabId::default(),
            selected_game: None,
            show_new_game_dialog: false,
            new_game_dialog,
            profiles_tab,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TabSelected(id) => {
                self.active_tab = id;

                Action::None
            }
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
            Message::ProfilesTab(message) => match self.profiles_tab.update(message) {
                profiles_tab::Action::None => Action::None,
                profiles_tab::Action::Run(task) => Action::Run(task.map(Message::ProfilesTab)),
                profiles_tab::Action::Create { name } => match &self.selected_game {
                    Some(game) => Action::Mutate(Mutation::CreateProfile {
                        game: game.clone(),
                        name,
                    }),
                    None => Action::None,
                },
                profiles_tab::Action::Mutate(mutation) => Action::Mutate(mutation),
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

        let body = {
            let games_sidebar = column![
                text(t!("game", { "count" => 2 })),
                rule::horizontal(1),
                scrollable(Column::with_children(model.games().iter().map(game_row))),
                space::vertical(),
                new_game_button
            ];

            let content_pane = if self.selected_game.is_some() {
                let tab_bar = row![
                    self.tab_button(TabId::Overview),
                    self.tab_button(TabId::Profiles),
                ];
                let tab_view: Element<'_, Message> = match self.active_tab {
                    TabId::Overview => column![button(text(t!("activate"))).on_press(
                        Message::ActivateButtonPressed(self.selected_game.clone().unwrap())
                    )]
                    .into(),
                    TabId::Profiles => self.profiles_tab.view(model).map(Message::ProfilesTab),
                };

                column![tab_bar, tab_view]
            } else {
                column![text("No selected game")]
            };

            row![
                games_sidebar.width(Length::FillPortion(1)),
                content_pane.width(Length::FillPortion(2))
            ]
            .padding(20)
        };

        let content = column![title_bar, body].into();

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

    fn tab_button(&self, tab: TabId) -> Element<'_, Message> {
        let label = match tab {
            TabId::Overview => t!("library-manager_overview"),
            TabId::Profiles => t!("profile", { "count" => 2 }),
        };
        let style = if self.active_tab == tab {
            button::primary
        } else {
            button::subtle
        };

        button(text(label))
            .on_press(Message::TabSelected(tab))
            .style(style)
            .into()
    }
}

pub fn game_row<'a>(row: &GameRow) -> Element<'a, Message> {
    let content = row![text(row.name.clone()), space::horizontal()];

    // if game == active_game {
    //     content = content.push(Icon::Check);
    // }
    //
    // let style = if Some(&self.game) == selected_game.as_ref() {
    //     button::primary
    // } else {
    //     button::subtle
    // };

    button(content)
        .width(Length::Fill)
        // .style(style)
        .on_press(Message::GameRowSelected(row.handle().clone()))
        .into()
}
