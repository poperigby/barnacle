use barnacle_gui::Component;
use barnacle_lib::Repository;
use iced::{
    Element, Task,
    widget::{button, column, container, row},
};

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
    games_tab: games_tab::Tab,
}

impl Component for LibraryManager {
    type Message = Message;

    fn new(repo: Repository) -> (Self, Task<Message>) {
        let (games_tab, games_task) = games_tab::Tab::new(repo.clone());
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(id) => {
                self.active_tab = id;
                Task::none()
            }
            Message::GamesTab(msg) => self.games_tab.update(msg).map(Message::GamesTab),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(column![
            row![button("Games").on_press(Message::TabSelected(TabId::Games))],
            match self.active_tab {
                TabId::Games => self.games_tab.view().map(Message::GamesTab),
            },
        ])
        .width(1000)
        .height(800)
        .style(container::rounded_box)
        .into()
    }
}
