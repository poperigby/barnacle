use barnacle_lib::repository::{DeployKind, Game};
use iced::{
    Element, Task,
    widget::{button, column, combo_box, container, row, space, text, text_input},
};
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DeployKindSelected(DeployKind),
    CancelPressed,
    ConfirmPressed,
    GameEdited,
}

pub struct EditDialog {
    game: Option<Game>,
    name: String,
    deploy_kind: Option<DeployKind>,
    deploy_kind_state: combo_box::State<DeployKind>,
}

impl EditDialog {
    /// Load a new [`Game`] for editing.
    pub fn load(&mut self, game: Game) {
        self.game = Some(game.clone());

        self.name = game.name().unwrap();
        self.deploy_kind = Some(game.deploy_kind().unwrap());
    }
}

impl EditDialog {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                game: None,
                name: "".into(),
                deploy_kind: None,
                deploy_kind_state: combo_box::State::new(DeployKind::iter().collect()),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NameInput(content) => {
                self.name = content;
                Task::none()
            }
            Message::DeployKindSelected(kind) => {
                self.deploy_kind = Some(kind);
                Task::none()
            }
            Message::CancelPressed => Task::none(),
            Message::ConfirmPressed => {
                let mut game = self.game.clone();

                let new_name = self.name.clone();
                let new_deploy_kind = self.deploy_kind.unwrap();

                // Reset dialog state
                self.name.clear();
                self.deploy_kind = None;

                Task::perform(
                    async move {
                        game.as_mut().unwrap().set_name(&new_name).unwrap();
                        game.as_mut()
                            .unwrap()
                            .set_deploy_kind(new_deploy_kind)
                            .unwrap();
                    },
                    |_| Message::GameEdited,
                )
            }
            Message::GameEdited => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![
            row![
                text("Name: "),
                text_input("Name", &self.name).on_input(Message::NameInput),
            ],
            row![
                text("Deploy kind: "),
                combo_box(
                    &self.deploy_kind_state,
                    "Select a deploy kind",
                    self.deploy_kind.as_ref(),
                    Message::DeployKindSelected
                ),
            ],
            space::vertical(),
            row![
                space::horizontal(),
                button("Cancel").on_press(Message::CancelPressed),
                button("Confirm").on_press(Message::ConfirmPressed),
            ],
        ])
        .padding(20)
        .width(400)
        .height(600)
        .style(container::rounded_box)
        .into()
    }
}
