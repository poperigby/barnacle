use barnacle_gui::model::{ItemGroupRecord, ItemRecord, Mutation};
use iced::{
    Element, Length, Task,
    widget::{checkbox, column, scrollable, table, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleItemEntry {
        entry_record: ItemRecord,
        enabled: bool,
    },
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct ItemGroupTab {}

impl ItemGroupTab {
    pub fn update(message: Message) -> Action {
        Action::None
    }

    pub fn view(group: &ItemGroupRecord) -> Element<'_, Message> {
        let columns = [
            table::column(text("Name"), |record: ItemRecord| {
                text(record.relative_path.to_str().unwrap().to_string())
            }),
            table::column(text("Status"), |entry_record: ItemRecord| {
                checkbox(entry_record.enabled).on_toggle(move |state| Message::ToggleItemEntry {
                    entry_record: entry_record.clone(),
                    enabled: state,
                })
            }),
        ];

        column![scrollable(
            table(columns, group.entries.clone()).width(Length::Fill)
        )]
        .into()
    }
}
