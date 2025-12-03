use barnacle_lib::{Repository, repository::DeployKind};
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
    CreatePressed,
    GameCreated,
}

pub struct NewDialog {
    repo: Repository,
    name: String,
    deploy_kind: Option<DeployKind>,
    deploy_kind_state: combo_box::State<DeployKind>,
}

impl NewDialog {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
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

    /// Reset the dialog state
    pub fn clear(&mut self) {
        self.name.clear();
        self.deploy_kind = None;
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
            Message::CreatePressed => {
                let repo = self.repo.clone();
                let name = self.name.clone();
                let deploy_kind = self.deploy_kind.unwrap();

                self.clear();

                Task::perform(
                    async move {
                        repo.add_game(&name, deploy_kind).unwrap();
                    },
                    |_| Message::GameCreated,
                )
            }
            Message::GameCreated => Task::none(),
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
                button("Create").on_press(Message::CreatePressed),
            ],
        ])
        .padding(20)
        .width(400)
        .height(600)
        .style(container::rounded_box)
        .into()
    }
}
