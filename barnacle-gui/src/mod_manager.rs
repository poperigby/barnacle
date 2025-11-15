use barnacle_lib::{GameId, ProfileId, ProfileMod, db::Database};
use iced::{
    Element, Task,
    widget::{button, column, combo_box, row},
};

#[derive(Debug, Clone)]
pub enum ModManagerMessage {
    ModsLoaded(Vec<ProfileMod>),
}

#[derive(Debug)]
pub struct ModManagerPage {
    db: Database,
    current_profile: Option<ProfileId>,
}

impl ModManagerPage {
    pub fn new(db: Database) -> Self {
        Self {
            db: db.clone(),
            current_profile: None,
        }
        // Task::perform(
        //     async move {
        //         let current_profile = state.current_profile().await.unwrap().unwrap();
        //
        //         state.profile_mods(current_profile).await.unwrap()
        //     },
        //     ModManagerMessage::ModsLoaded,
        // ),
    }

    pub fn update(&mut self, _message: ModManagerMessage) -> Task<ModManagerMessage> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, ModManagerMessage> {
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
