use barnacle_gui::{
    TranslationElementExt,
    model::{ModEntryRecord, Mutation},
};
use barnacle_l10n::Translations;
use iced::{
    Element, Point,
    widget::{button, column, container},
};

#[derive(Debug, Clone)]
pub enum Message {
    DeleteButtonPressed,
}

#[derive(Debug)]
pub enum Action {
    Mutate(Mutation),
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub position: Point,
    record: ModEntryRecord,
}

impl ContextMenu {
    pub fn new(position: Point, record: ModEntryRecord) -> Self {
        Self { position, record }
    }

    pub fn view<'a>(&self, t: &Translations) -> Element<'a, Message> {
        let contents = column![
            button(t.mods_tab_context_menu_delete().element())
                .on_press(Message::DeleteButtonPressed)
        ];

        container(contents).style(container::rounded_box).into()
    }

    pub(crate) fn update(&self, message: Message) -> Action {
        match message {
            Message::DeleteButtonPressed => {
                Action::Mutate(Mutation::DeleteModEntry(self.record.object().clone()))
            }
        }
    }
}
