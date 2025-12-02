use barnacle_gui::Component;
use barnacle_lib::{
    Repository,
    repository::{DeployKind, Game},
};
use iced::{
    Element, Task,
    widget::{button, column, combo_box, container, row, space, text, text_input},
};
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DeployKindSelected(DeployKind),
    ConfirmPressed,
    GameEdited,
}

pub struct EditDialog {
    repo: Repository,
    name: String,
    deploy_kind: Option<DeployKind>,
    deploy_kind_state: combo_box::State<DeployKind>,
}

impl EditDialog {
    /// Load a new [`Game`] for editing.
    pub fn load(&mut self, game: Game) {
        self.name = game.name().unwrap().into();
        self.deploy_kind = Some(game.deploy_kind().unwrap());
    }
}

impl Component for EditDialog {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Self::Message>) {
        (
            Self {
                repo,
                name: "".into(),
                deploy_kind: None,
                deploy_kind_state: combo_box::State::new(DeployKind::iter().collect()),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NameInput(content) => {
                self.name = content;
                Task::none()
            }
            Message::DeployKindSelected(kind) => {
                self.deploy_kind = Some(kind);
                Task::none()
            }
            Message::ConfirmPressed => {
                let repo = self.repo.clone();
                let name = self.name.clone();
                let deploy_kind = self.deploy_kind.unwrap();

                // Reset dialog state
                self.name.clear();
                self.deploy_kind = None;

                Task::perform(
                    async move {
                        repo.add_game(&name, deploy_kind).unwrap();
                    },
                    |_| Message::GameEdited,
                )
            }
            Message::GameEdited => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
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
