use barnacle_lib::Repository;
use iced::{Element, Task, widget::container};
use iced_aw::{TabLabel, Tabs};

use crate::{Component, library_manager::games_tab::GamesTab};

mod games_tab;

const TAB_PADDING: u16 = 16;

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    // Components
    GamesTab(games_tab::Message),
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum TabId {
    #[default]
    Games,
}

pub struct LibraryManager {
    repo: Repository,
    active_tab: TabId,
    // Components
    games_tab: GamesTab,
}

impl Component for LibraryManager {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let (games_tab, games_task) = GamesTab::new(repo.clone());
        // let (profiles_tab, profiles_task) = ProfilesTab::new(repo.clone());
        // let (mods_tab, mods_task) = ModsTab::new(repo.clone());
        // let (tools_tab, tools_task) = ToolsTab::new(repo.clone());

        let tasks = Task::batch([games_task.map(Message::GamesTab)]);

        (
            Self {
                repo: repo.clone(),
                active_tab: TabId::default(),
                games_tab,
            },
            tasks,
        )
    }

    fn update(&mut self, message: Message) -> Task<Self::Message> {
        match message {
            Message::TabSelected(id) => {
                self.active_tab = id;
                Task::none()
            }
            Message::GamesTab(msg) => self.games_tab.update(msg).map(Message::GamesTab),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(
            Tabs::new(Message::TabSelected)
                .push(
                    TabId::Games,
                    TabLabel::Text("Games".into()),
                    self.games_tab.view().map(Message::GamesTab),
                )
                .set_active_tab(&self.active_tab),
        )
        .width(1000)
        .height(800)
        .style(container::rounded_box)
        .into()
    }
}
