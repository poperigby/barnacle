use barnacle_gui::{
    model::{ModEntryRecord, Model, Mutation},
    persistence::state::UiStateStore,
    widgets::table::{self, table},
};
use iced::{
    Element, Length, Point, Task,
    widget::{checkbox, column, scrollable, text},
};

pub mod add_mod_menu;

#[derive(Debug, Clone)]
pub enum Message {
    ModEntryToggled {
        entry_record: ModEntryRecord,
        enabled: bool,
    },
    ModEntryDeleted(ModEntryRecord),
    ShowContextMenu {
        index: usize,
        starting_pos: Point,
    },
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct ModsTab {
    ui_state: UiStateStore,
}

impl ModsTab {
    pub fn new(ui_state: UiStateStore) -> Self {
        Self { ui_state }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ModEntryDeleted(entry) => {
                Action::Mutate(Mutation::DeleteModEntry(entry.object()))
            }
            Message::ModEntryToggled {
                entry_record,
                enabled,
            } => Action::Mutate(Mutation::SetModEntryEnabled {
                entry: entry_record.object(),
                enabled,
            }),
            Message::ShowContextMenu {
                index,
                starting_pos,
            } => {
                dbg!(index, starting_pos);
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Message> {
        let columns = [
            table::column(text("Name"), |entry_record: ModEntryRecord| {
                text(entry_record.name.clone())
            }),
            table::column(text("Status"), |entry_record: ModEntryRecord| {
                checkbox(entry_record.enabled).on_toggle(move |state| Message::ModEntryToggled {
                    entry_record: entry_record.clone(),
                    enabled: state,
                })
            }),
        ];

        column![scrollable(
            table(columns, model.mod_entries())
                .width(Length::Fill)
                .on_right_click_row(|index, cursor_pos| Message::ShowContextMenu {
                    index,
                    starting_pos: cursor_pos
                })
        )]
        .into()
    }
}
