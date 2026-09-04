use barnacle_lib::repository::DeployKind;
use fluent_i18n::t;
use iced::{
    Element, Task,
    widget::{button, column, combo_box, container, row, space, text, text_input},
};
use strum::IntoEnumIterator;

pub const ID: &str = "new_game_dialog";

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DeployKindSelected(DeployKind),
    CancelPressed,
    CreatePressed,
}

#[derive(Debug)]
pub struct NewGame {
    pub name: String,
    pub deploy_kind: DeployKind,
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    CreateGame(NewGame),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    name: String,
    deploy_kind: Option<DeployKind>,
    deploy_kind_state: combo_box::State<DeployKind>,
}

impl Dialog {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
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

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NameInput(content) => {
                self.name = content;
                Action::None
            }
            Message::DeployKindSelected(kind) => {
                self.deploy_kind = Some(kind);
                Action::None
            }
            Message::CancelPressed => {
                self.clear();
                Action::Cancel
            }
            Message::CreatePressed => {
                let name = self.name.clone();
                let deploy_kind = self.deploy_kind.clone().unwrap();

                self.clear();

                Action::CreateGame(NewGame { name, deploy_kind })
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(column![
            row![
                text(t!("name")),
                text_input("...", &self.name).on_input(Message::NameInput),
            ],
            row![
                text(t!("library-manager_new-game-dialog_deploy-kind")),
                combo_box(
                    &self.deploy_kind_state,
                    "...",
                    self.deploy_kind.as_ref(),
                    Message::DeployKindSelected
                ),
            ],
            space::vertical(),
            row![
                space::horizontal(),
                button(text(t!("cancel"))).on_press(Message::CancelPressed),
                button(text(t!("create")))
                    .on_press_maybe(self.validate().then_some(Message::CreatePressed)),
            ],
        ])
        .width(400)
        .height(500)
        .padding(20)
        .style(container::rounded_box)
        .into()
    }

    fn validate(&self) -> bool {
        !self.name.is_empty() && self.deploy_kind.is_some()
    }
}
