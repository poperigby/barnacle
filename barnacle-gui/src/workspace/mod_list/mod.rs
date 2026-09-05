use crate::{
    persistence::state::UiStateStore,
    workspace::mod_list::state::{SortColumn, SortState},
};
use barnacle_lib::repository::{Profile, handles::ModEntry};
use iced::{
    Element, Length, Task,
    widget::{button, checkbox, column, row, scrollable, table, text},
};
use iced_aw::Spinner;

pub mod state;

#[derive(Debug, Clone)]
pub enum Message {
    StateChanged(State),
    SortChanged(SortColumn),
    ToggleModEntry(ModEntryRow, bool),
    ModEntryToggled,
    ModEntryDeleted(ModEntryRow),
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
    Ready(Vec<ModEntryRow>),
}

#[derive(Debug, Clone)]
pub struct ModList {
    ui_state: UiStateStore,
    state: State,
    sort: SortState,
}

impl ModList {
    pub fn new(ui_state: UiStateStore, profile: Profile) -> (Self, Task<Message>) {
        let initial_sort_state = ui_state.mod_list_sort_state();
        let list = Self {
            ui_state,
            state: State::Loading,
            sort: initial_sort_state,
        };
        let task = list.load(profile);

        (list, task)
    }

    pub fn load(&self, profile: Profile) -> Task<Message> {
        Task::perform(
            async move {
                let mut rows = Vec::new();
                for mod_entry in profile.mod_entries().await.unwrap() {
                    let name = mod_entry.name().await.unwrap();
                    let enabled = mod_entry.enabled().await.unwrap();
                    rows.push(ModEntryRow::new(mod_entry, name, enabled));
                }

                rows
            },
            |rows| Message::StateChanged(State::Ready(rows)),
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
                self.ui_state.set_mod_list_sort_state(self.sort);
                Action::None
            }
            Message::ToggleModEntry(entry_row, state) => {
                let entry = entry_row.entry.clone();

                Action::Run(Task::perform(
                    async move {
                        entry.set_enabled(state).await.unwrap();
                    },
                    |_| Message::ModEntryToggled,
                ))
            }
            Message::ModEntryDeleted(entry) => {
                println!("Deletion of {:?}", entry);
                // entry.remove().unwrap();
                Action::None
            }
            Message::ModEntryToggled => Action::None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => Spinner::new().into(),
            State::Error(e) => text(e).into(),
            State::Ready(mod_entries_rows) => {
                let columns = [
                    table::column(
                        column_header("Name", &self.sort, SortColumn::Name),
                        |entry_row: ModEntryRow| text(entry_row.name),
                    ),
                    table::column(
                        column_header("Cateogry", &self.sort, SortColumn::Category),
                        |entry_row: ModEntryRow| text("Category"),
                    ),
                    table::column(text("Status"), |entry_row: ModEntryRow| {
                        checkbox(entry_row.enabled).on_toggle(move |state| {
                            Message::ToggleModEntry(entry_row.clone(), state)
                        })
                    }),
                ];

                column![scrollable(
                    table(columns, mod_entries_rows.clone()).width(Length::Fill)
                )]
                .into()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModEntryRow {
    entry: ModEntry,
    name: String,
    enabled: bool,
}

impl ModEntryRow {
    pub fn new(entry: ModEntry, name: String, enabled: bool) -> Self {
        Self {
            entry,
            name,
            enabled,
        }
    }

    // pub fn view<'a>(&self) -> Element<'a, Message> {}
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
