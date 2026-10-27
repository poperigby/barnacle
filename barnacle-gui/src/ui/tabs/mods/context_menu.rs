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
    DeleteEntryButtonPressed,
    DeleteModButtonPressed,
}

pub enum Action {
    Mutate(Mutation<Message>),
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
            button(t.mods_tab_context_menu_delete_entry().element())
                .on_press(Message::DeleteEntryButtonPressed),
            button(t.mods_tab_context_menu_delete_mod().element())
                .on_press(Message::DeleteModButtonPressed)
        ];

        container(contents).style(container::rounded_box).into()
    }

    pub(crate) fn update(&self, message: Message) -> Action {
        match message {
            Message::DeleteEntryButtonPressed => {
                Action::Mutate(Mutation::delete_mod_entry(self.record.object().clone()))
            }
            Message::DeleteModButtonPressed => {
                Action::Mutate(Mutation::delete_mod(self.record.mod_object.clone()))
            }
        }
    }
}
