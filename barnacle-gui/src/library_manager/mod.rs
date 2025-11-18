use barnacle_lib::Repository;
use iced::{
    Alignment, Element, Length, Task,
    alignment::{Horizontal, Vertical},
    widget::{column, container, text},
};
use iced_aw::{TabLabel, Tabs};

use crate::library_manager::{
    games_tab::GamesTab, mods_tab::ModsTab, profiles_tab::ProfilesTab, tools_tab::ToolsTab,
};

mod games_tab;
mod mods_tab;
mod profiles_tab;
mod tools_tab;

const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 16;

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    // Components
    GamesTab(games_tab::Message),
    ProfilesTab(profiles_tab::Message),
    ModsTab(mods_tab::Message),
    ToolsTab(tools_tab::Message),
}

// Action the ModList wants the App to perform for it
pub enum Action {
    Task(Task<Message>),
    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TabId {
    Games,
    Profiles,
    Mods,
    Tools,
}

pub struct LibraryManager {
    repo: Repository,
    active_tab: TabId,
    // Components
    games_tab: GamesTab,
    profiles_tab: ProfilesTab,
    mods_tab: ModsTab,
    tools_tab: ToolsTab,
}

impl LibraryManager {
    pub fn new(repo: Repository) -> (Self, Task<Message>) {
        (
            Self {
                repo,
                active_tab: TabId::Games,
                games_tab: GamesTab {},
                profiles_tab: ProfilesTab {},
                mods_tab: ModsTab {},
                tools_tab: ToolsTab {},
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TabSelected(id) => {
                self.active_tab = id;
                Action::None
            }
            Message::GamesTab(msg) => {
                self.games_tab.update(msg);
                Action::None
            }
            Message::ProfilesTab(msg) => {
                self.profiles_tab.update(msg);
                Action::None
            }
            Message::ModsTab(msg) => {
                self.mods_tab.update(msg);
                Action::None
            }
            Message::ToolsTab(msg) => {
                self.tools_tab.update(msg);
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            Tabs::new(Message::TabSelected)
                .push(
                    TabId::Games,
                    self.games_tab.tab_label(),
                    self.games_tab.view().map(Message::GamesTab),
                )
                .push(
                    TabId::Profiles,
                    self.profiles_tab.tab_label(),
                    self.profiles_tab.view().map(Message::ProfilesTab),
                )
                .push(
                    TabId::Mods,
                    self.mods_tab.tab_label(),
                    self.mods_tab.view().map(Message::ModsTab),
                )
                .push(
                    TabId::Tools,
                    self.tools_tab.tab_label(),
                    self.tools_tab.view().map(Message::ToolsTab),
                )
                .set_active_tab(&self.active_tab),
        )
        .width(800)
        .height(900)
        .style(container::rounded_box)
        .into()
    }
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        container(
            column![text(self.title()).size(HEADER_SIZE), self.content()]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding(TAB_PADDING)
        .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}
