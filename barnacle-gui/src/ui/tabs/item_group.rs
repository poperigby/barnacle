use barnacle_gui::{
    TranslationElementExt,
    model::{ItemGroupRecord, ItemRecord, Mutation},
};
use barnacle_l10n::Translations;
use barnacle_lib::repository::item_entries::{ItemEntry, ItemGroup};
use iced::{
    Element, Length, Task,
    widget::{button, checkbox, column, scrollable, space, table, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleItemEntry { entry: ItemEntry, enabled: bool },
    AutoSortButtonPressed { group: ItemGroup },
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
        match message {
            Message::ToggleItemEntry { entry, enabled } => {
                Action::Mutate(Mutation::SetItemEntryEnabled { entry, enabled })
            }
            Message::AutoSortButtonPressed { group } => {
                Action::Mutate(Mutation::AutoSortItemGroup { group })
            }
        }
    }

    pub fn view<'a>(
        group: &'a ItemGroupRecord,
        translations: &'a Translations,
    ) -> Element<'a, Message> {
        let action_bar: Element<'a, Message> = if group.object.supports_auto_sort() {
            button(translations.items_tab_sort_button().element())
                .on_press(Message::AutoSortButtonPressed {
                    group: group.object.clone(),
                })
                .into()
        } else {
            space().into()
        };

        let columns = [
            table::column(text("Name"), |record: ItemRecord| {
                text(record.relative_path.to_str().unwrap().to_string())
            }),
            table::column(text("Status"), |entry_record: ItemRecord| {
                checkbox(entry_record.enabled).on_toggle(move |state| Message::ToggleItemEntry {
                    entry: entry_record.object.clone(),
                    enabled: state,
                })
            }),
        ];

        column![
            action_bar,
            scrollable(table(columns, group.entries.clone()).width(Length::Fill))
        ]
        .into()
    }
}
