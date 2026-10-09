use barnacle_l10n::Translations;
use barnacle_lib::games::Kind;
use iced::{
    Element, Task,
    widget::{button, column, combo_box, container, row, space, text, text_input},
};
use strum::IntoEnumIterator;

use barnacle_gui::{TranslationElementExt, model::Mutation};

pub const ID: &str = "new_game_dialog";

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DeployKindSelected(Kind),
    CancelPressed,
    CreatePressed,
}

#[derive(Debug)]
pub struct NewGame {
    pub name: String,
    pub deploy_kind: Kind,
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct Menu {
    name: String,
    deploy_kind: Option<Kind>,
    deploy_kind_state: combo_box::State<Kind>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            name: "".into(),
            deploy_kind: None,
            deploy_kind_state: combo_box::State::new(Kind::iter().collect()),
        }
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

                Action::Mutate(Mutation::CreateGame { name, deploy_kind })
            }
        }
    }

    pub fn view(&self, translations: &Translations) -> Element<'_, Message> {
        container(column![
            row![
                translations.name().element(),
                text_input("...", &self.name).on_input(Message::NameInput),
            ],
            row![
                translations
                    .library_manager_new_game_dialog_deploy_kind()
                    .element(),
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
                button(translations.cancel().element()).on_press(Message::CancelPressed),
                button(translations.create().element())
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
