use crate::{
    model::{ModEntryRow, Model, Mutation},
    persistence::state::UiStateStore,
    ui::mod_list::state::{SortColumn, SortState},
};
use iced::{
    Element, Length, Task,
    widget::{button, checkbox, column, row, scrollable, table, text},
};

pub mod state;

#[derive(Debug, Clone)]
pub enum Message {
    SortChanged(SortColumn),
    ToggleModEntry {
        entry_row: ModEntryRow,
        enabled: bool,
    },
    ModEntryToggled,
    ModEntryDeleted(ModEntryRow),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct ModList {
    ui_state: UiStateStore,
    sort: SortState,
}

impl ModList {
    pub fn new(ui_state: UiStateStore) -> Self {
        let initial_sort_state = ui_state.mod_list_sort_state();
        Self {
            ui_state,
            sort: initial_sort_state,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::SortChanged(column) => {
                self.sort = self.sort.toggle(column);
                self.ui_state.set_mod_list_sort_state(self.sort);
                Action::None
            }
            Message::ModEntryDeleted(entry) => {
                println!("Deletion of {:?}", entry);
                // entry.remove().unwrap();
                Action::None
            }
            Message::ModEntryToggled => Action::None,
            Message::ToggleModEntry { entry_row, enabled } => {
                Action::Mutate(Mutation::SetModEntryEnabled {
                    entry: entry_row.handle(),
                    enabled,
                })
            }
        }
    }

    pub fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Message> {
        let columns = [
            table::column(
                column_header("Name", &self.sort, SortColumn::Name),
                |entry_row: &ModEntryRow| text(entry_row.name.clone()),
            ),
            table::column(
                column_header("Cateogry", &self.sort, SortColumn::Category),
                |_entry_row: &ModEntryRow| text("Category"),
            ),
            table::column(text("Status"), |entry_row: &ModEntryRow| {
                checkbox(entry_row.enabled).on_toggle(move |state| Message::ToggleModEntry {
                    entry_row: entry_row.clone(),
                    enabled: state,
                })
            }),
        ];

        column![scrollable(
            table(columns, model.mod_entries()).width(Length::Fill)
        )]
        .into()
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
