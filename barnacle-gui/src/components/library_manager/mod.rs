use crate::{
    components::library_manager::{new_game_dialog::NewGame, profiles_tab::new_dialog::NewProfile},
    icons::icon,
    modal,
};
use barnacle_lib::{
    Repository,
    repository::{Game, Profile},
};
use fluent_i18n::t;
use iced::{
    Element, Length, Task,
    widget::{Column, button, column, container, row, rule, scrollable, space, text},
};
use iced_aw::Spinner;

pub mod new_game_dialog;
pub mod profiles_tab;

#[derive(Debug, Clone)]
pub enum Message {
    StateChanged(State),
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
    CreateGame(NewGame),
    DeleteGame(Game),
    ActivateGame(Game),
    CreateProfile { game: Game, new_profile: NewProfile },
    DeleteProfile(Profile),
    Close,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum TabId {
    Overview,
    #[default]
    Profiles,
}

#[derive(Debug, Clone)]
pub enum State {
    Loading,
    Error(String),
    NoGames,
    Loaded {
        active_game: Game,
        games_rows: Vec<GameRow>,
    },
}

#[derive(Debug, Clone)]
pub struct LibraryManager {
    repo: Repository,
    state: State,
    // State
    active_tab: TabId,
    selected_game: Option<Game>,
    show_new_game_dialog: bool,
    // Components
    new_game_dialog: new_game_dialog::Dialog,
    profiles_tab: profiles_tab::Tab,
}

impl LibraryManager {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        let (new_game_dialog, new_game_dialog_task) = new_game_dialog::Dialog::new();
        let profiles_tab = profiles_tab::Tab::new(repo.clone());

        (
            Self {
                repo: repo.clone(),
                state: State::Loading,
                active_tab: TabId::default(),
                selected_game: None,
                show_new_game_dialog: false,
                new_game_dialog,
                profiles_tab,
            },
            Task::batch([
                new_game_dialog_task.map(Message::NewGameDialog),
                load_state(repo),
            ]),
        )
    }

    pub fn refresh(&self) -> Task<Message> {
        load_state(self.repo.clone())
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::StateChanged(state) => {
                self.state = state;
                match &self.state {
                    State::Loaded { active_game, .. } => {
                        // If there isn't already a selected game, we can set it to the active game
                        let selected_game = self.selected_game.get_or_insert(active_game.clone());

                        // We only want to load the tab contents if we have a selected_game
                        Action::Run(
                            self.profiles_tab
                                .refresh(selected_game)
                                .map(Message::ProfilesTab),
                        )
                    }
                    _ => Action::None,
                }
            }
            Message::TabSelected(id) => {
                self.active_tab = id;
                Action::None
            }
            Message::CloseButtonPressed => Action::Close,
            Message::NewGameButtonPressed => {
                self.show_new_game_dialog = true;
                Action::None
            }
            Message::ActivateButtonPressed(game) => Action::ActivateGame(game),
            Message::GameRowSelected(game) => {
                self.selected_game = Some(game.clone());
                Action::Run(self.profiles_tab.refresh(&game).map(Message::ProfilesTab))
            }
            Message::NewGameDialog(message) => match self.new_game_dialog.update(message) {
                new_game_dialog::Action::None => Action::None,
                new_game_dialog::Action::Run(task) => Action::Run(task.map(Message::NewGameDialog)),
                new_game_dialog::Action::CreateGame(new_game) => {
                    self.show_new_game_dialog = false;
                    Action::CreateGame(new_game)
                }
                new_game_dialog::Action::Cancel => {
                    self.show_new_game_dialog = false;
                    Action::None
                }
            },
            Message::ProfilesTab(message) => match self.profiles_tab.update(message) {
                // TODO: Do top-level if let Some(selected_game)
                profiles_tab::Action::None => Action::None,
                profiles_tab::Action::Run(task) => Action::Run(task.map(Message::ProfilesTab)),
                profiles_tab::Action::Refresh => {
                    if let Some(selected_game) = &self.selected_game {
                        Action::Run(
                            self.profiles_tab
                                .refresh(selected_game)
                                .map(Message::ProfilesTab),
                        )
                    } else {
                        Action::None
                    }
                }
                profiles_tab::Action::Create(new_profile) => {
                    if let Some(selected_game) = &self.selected_game {
                        Action::CreateProfile {
                            game: selected_game.clone(),
                            new_profile,
                        }
                    } else {
                        Action::None
                    }
                }
                profiles_tab::Action::Delete(profile) => Action::DeleteProfile(profile),
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title_bar = row![
            text(t!("library-manager_title")),
            space::horizontal(),
            button(icon("close")).on_press(Message::CloseButtonPressed)
        ];

        let new_game_button = button(row![
            icon("plus"),
            text(t!("library-manager_new-game", { "count" => 1 }))
        ])
        .on_press(Message::NewGameButtonPressed);

        let body: Element<'_, Message> = match &self.state {
            State::Loading => Spinner::new().into(),
            State::Error(e) => text(e).into(),
            State::NoGames => column![text("No games"), new_game_button].into(),
            State::Loaded {
                active_game,
                games_rows: games,
            } => {
                let game_rows = games
                    .iter()
                    .map(|row| row.view(active_game, &self.selected_game));

                let games_sidebar = column![
                    text(t!("game", { "count" => 2 })),
                    rule::horizontal(1),
                    scrollable(Column::with_children(game_rows)),
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
                        TabId::Profiles => self.profiles_tab.view().map(Message::ProfilesTab),
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
                .into()
            }
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

fn load_state(repo: Repository) -> Task<Message> {
    let repo = repo.clone();
    Task::perform(
        async move {
            let active_game = repo.active_game().await.unwrap();

            let mut rows = Vec::new();
            for game in repo.games().await.unwrap() {
                let name = game.name().await.unwrap();
                rows.push(GameRow::new(game, name));
            }

            if !rows.is_empty() {
                State::Loaded {
                    active_game: active_game.unwrap(),
                    games_rows: rows,
                }
            } else {
                State::NoGames
            }
        },
        Message::StateChanged,
    )
}

#[derive(Debug, Clone)]
pub struct GameRow {
    game: Game,
    name: String,
}

impl GameRow {
    pub fn new(game: Game, name: String) -> Self {
        Self { game, name }
    }

    pub fn view<'a>(
        &self,
        active_game: &'a Game,
        selected_game: &'a Option<Game>,
    ) -> Element<'a, Message> {
        let mut content = row![text(self.name.clone()), space::horizontal()];

        if &self.game == active_game {
            content = content.push(icon("check"));
        }

        let style = if Some(&self.game) == selected_game.as_ref() {
            button::primary
        } else {
            button::subtle
        };

        button(content)
            .width(Length::Fill)
            .style(style)
            .on_press(Message::GameRowSelected(self.game.clone()))
            .into()
    }
}
