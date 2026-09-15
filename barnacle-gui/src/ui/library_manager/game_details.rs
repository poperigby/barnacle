use barnacle_gui::{
    icons::Icon,
    model::{GameItem, Model, Mutation, ProfileItem},
};
use barnacle_lib::{Game, repository::Profile};
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, space, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    ActivateGameButtonPressed(Game),
    NewProfileButtonPressed,
    EditProfileButtonPressed {
        profile: Profile,
        // The name the [`Profile`] has when the edit dialog is opened
        initial_name: String,
    },
    DeleteProfileButtonPressed(Profile),
}

pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    NewProfile,
    EditProfile {
        profile: Profile,
        initial_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct GameDetails {}

impl GameDetails {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ActivateGameButtonPressed(game) => {
                Action::Mutate(Mutation::ActivateGame(game))
            }
            Message::DeleteProfileButtonPressed(profile) => {
                Action::Mutate(Mutation::DeleteProfile(profile))
            }
            Message::NewProfileButtonPressed => Action::NewProfile,
            Message::EditProfileButtonPressed {
                profile,
                initial_name,
            } => Action::EditProfile {
                profile,
                initial_name,
            },
        }
    }
    pub fn view(&self, model: &Model, selected_game: &Option<GameItem>) -> Element<'_, Message> {
        let game = match selected_game {
            Some(game) => game,
            None => return column![text("No game selected")].into(),
        };

        let active_control: Element<'_, Message> = if game.active {
            text("Current").into()
        } else {
            button("Activate")
                .on_press(Message::ActivateGameButtonPressed(game.object()))
                .into()
        };

        let top_row = row![text(game.name.clone()), space::horizontal(), active_control];

        let profile_rows: Vec<_> = game.profiles.iter().map(profile_row).collect();
        let profiles_view = column![
            text("Profiles"),
            scrollable(column(profile_rows)),
            button(row![Icon::Plus, text("New Profile")])
                .on_press(Message::NewProfileButtonPressed)
        ];

        column![top_row, profiles_view]
            .width(Length::FillPortion(2))
            .into()
    }
}

fn profile_row<'a>(row: &ProfileItem) -> Element<'a, Message> {
    container(
        row![
            text(row.name.clone()),
            space::horizontal(),
            button(Icon::Edit),
            button(Icon::Delete)
                .on_press(Message::DeleteProfileButtonPressed(row.object().clone()))
        ]
        .padding(12),
    )
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}
