use barnacle_gui::{
    model::{ModEntryItem, Model, Mutation},
    persistence::state::UiStateStore,
};
use iced::{
    Element, Length, Task,
    widget::{checkbox, column, scrollable, table, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleModEntry {
        entry_item: ModEntryItem,
        enabled: bool,
    },
    ModEntryToggled,
    ModEntryDeleted(ModEntryItem),
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
}

impl ModList {
    pub fn new(ui_state: UiStateStore) -> Self {
        Self { ui_state }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ModEntryDeleted(entry) => {
                println!("Deletion of {:?}", entry);
                // entry.remove().unwrap();
                Action::None
            }
            Message::ModEntryToggled => Action::None,
            Message::ToggleModEntry {
                entry_item,
                enabled,
            } => Action::Mutate(Mutation::SetModEntryEnabled {
                entry: entry_item.object(),
                enabled,
            }),
        }
    }

    pub fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Message> {
        let columns = [
            table::column(text("Name"), |entry_item: ModEntryItem| {
                text(entry_item.name.clone())
            }),
            table::column(text("Status"), |entry_item: ModEntryItem| {
                checkbox(entry_item.enabled).on_toggle(move |state| Message::ToggleModEntry {
                    entry_item: entry_item.clone(),
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
