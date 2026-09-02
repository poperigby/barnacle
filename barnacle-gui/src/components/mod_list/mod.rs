use crate::{
    components::mod_list::state::{ContextMenuState, SortColumn, SortState},
    config::Cfg,
};
use barnacle_lib::{
    Repository,
    repository::{Profile, handles::ModEntry},
};
use iced::{
    Element, Length, Point, Task,
    widget::{button, checkbox, column, row, scrollable, table, text},
};
use iced_aw::Spinner;
use sweeten::widget::mouse_area;
use tokio::task::spawn_blocking;

pub mod state;

#[derive(Debug, Clone)]
pub enum Message {
    StateChanged(State),
    SortChanged(SortColumn),
    ClickedOutContextMenu,
    ToggleModEntry(ModEntry, bool),
    ModEntryRightClicked(ModEntry, Point),
    ModEntryDeleted(ModEntry),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum State {
    Loading,
    Error(String),
    Loaded(Vec<ModEntry>),
}

pub struct ModList {
    repo: Repository,
    cfg: Cfg,
    state: State,
    sort: SortState,
    context_menu: Option<ContextMenuState>,
}

impl ModList {
    pub fn new(repo: Repository, cfg: Cfg) -> Self {
        Self {
            repo: repo.clone(),
            cfg,
            state: State::Loading,
            sort: SortState::default(),
            context_menu: None,
        }
    }

    pub fn refresh(&self, profile: &Profile) -> Task<Message> {
        let profile = profile.clone();
        Task::perform(
            async {
                spawn_blocking(move || State::Loaded(profile.mod_entries().unwrap()))
                    .await
                    .unwrap()
            },
            Message::StateChanged,
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::StateChanged(state) => {
                self.state = state;
                Action::None
            }
            Message::SortChanged(column) => {
                self.sort = self.sort.toggle(column);
                self.cfg.write().mod_list.sort_state = self.sort;
                Action::None
            }
            Message::ClickedOutContextMenu => {
                self.context_menu = None;
                Action::None
            }
            Message::ToggleModEntry(entry, state) => {
                // TODO: This should be async
                let entry = entry.clone();
                entry.set_enabled(state).unwrap();
                Action::None
            }
            Message::ModEntryRightClicked(entry, position) => {
                self.context_menu = Some(ContextMenuState::new(entry, position));
                Action::None
            }
            Message::ModEntryDeleted(entry) => {
                println!("Deletion of {:?}", entry);
                // entry.remove().unwrap();
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => Spinner::new().into(),
            State::Error(e) => text(e).into(),
            State::Loaded(mod_entries) => {
                let columns = [
                    table::column(
                        column_header("Name", &self.sort, SortColumn::Name),
                        |entry: ModEntry| {
                            mouse_area(text(entry.name().unwrap())).on_right_press(move |point| {
                                Message::ModEntryRightClicked(entry.clone(), point)
                            })
                        },
                    ),
                    table::column(
                        column_header("Cateogry", &self.sort, SortColumn::Category),
                        |entry: ModEntry| text("Category"),
                    ),
                    table::column(text("Status"), |entry: ModEntry| {
                        checkbox(entry.enabled().unwrap())
                            .on_toggle(move |state| Message::ToggleModEntry(entry.clone(), state))
                    }),
                ];

                column![scrollable(
                    table(columns, mod_entries.clone()).width(Length::Fill)
                )]
                .into()
            }
        }
    }
}

fn column_header<'a>(
    name: &'a str,
    sort_state: &'a SortState,
    column: SortColumn,
) -> Element<'a, Message> {
    button(row![text(name), sort_state.icon(column)])
        .style(button::subtle)
        .on_press(Message::SortChanged(column))
        .into()
}
