use crate::{
    components::mod_list::state::{ContextMenuState, SortColumn, SortState},
    config::Cfg,
    widgets::table::{self, table},
};
use barnacle_lib::repository::{Profile, models::ModEntry};
use iced::{
    Element, Length, Point, Task,
    widget::{button, checkbox, column, row, scrollable, text},
};
use iced_aw::Spinner;

pub mod state;

#[derive(Debug, Clone)]
pub enum Message {
    StateChanged(State),
    SortChanged(SortColumn),
    ClickedOutContextMenu,
    ToggleModEntry(ModEntry, bool),
    ModEntryToggled(ModEntry, bool),
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
    Loaded(Vec<ModEntryRow>),
}

pub struct ModList {
    cfg: Cfg,
    state: State,
    sort: SortState,
    context_menu: Option<ContextMenuState>,
}

impl ModList {
    pub fn new(cfg: Cfg) -> Self {
        Self {
            cfg,
            state: State::Loading,
            sort: SortState::default(),
            context_menu: None,
        }
    }

    pub fn refresh(&self, profile: &Profile) -> Task<Message> {
        let profile = profile.clone();
        Task::perform(
            async move {
                let entries = profile.mod_entries().await.unwrap();
                let mut rows = Vec::with_capacity(entries.len());
                for entry in entries {
                    rows.push(ModEntryRow {
                        name: entry.name().await.unwrap(),
                        enabled: entry.enabled().await.unwrap(),
                        entity: entry,
                    });
                }
                State::Loaded(rows)
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
            Message::ToggleModEntry(entry, state) => Action::Run(Task::perform(
                async move {
                    entry.set_enabled(state).await.unwrap();
                    (entry, state)
                },
                |(entry, state)| Message::ModEntryToggled(entry, state),
            )),
            Message::ModEntryToggled(entry, state) => {
                if let State::Loaded(rows) = &mut self.state
                    && let Some(row) = rows.iter_mut().find(|row| row.entity == entry)
                {
                    row.enabled = state;
                }
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
                        |entry: ModEntryRow| text(entry.name.clone()),
                    ),
                    table::column(
                        column_header("Cateogry", &self.sort, SortColumn::Category),
                        |_entry: ModEntryRow| text("Category"),
                    ),
                    table::column(text("Status"), |entry: ModEntryRow| {
                        checkbox(entry.enabled).on_toggle(move |state| {
                            Message::ToggleModEntry(entry.entity.clone(), state)
                        })
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

#[derive(Debug, Clone)]
pub struct ModEntryRow {
    entity: ModEntry,
    name: String,
    enabled: bool,
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
