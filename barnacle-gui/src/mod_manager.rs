use barnacle_lib::GameId;
use iced::{
    Element,
    widget::{button, column, combo_box, row},
};

use crate::Message;

#[derive(Debug, Clone)]
pub enum ModManagerMessage {}

#[derive(Debug)]
pub struct ModManagerPage;
// {
// games: combo_box::State<GameId>,
// current_game: GameId,
// };

impl ModManagerPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _message: ModManagerMessage) {}

    pub fn view(&self) -> Element<'_, Message> {
        column![
            row![
                "Game:",
                button("Play"),
                "Profile:",
                button("Library Manager"),
                button("Settings"),
                button("Notifications")
            ],
            row![button("Install"), button("Deploy"), button("Tools")]
        ]
        .into()
    }
}
