use barnacle_gui::{
    model::{ModEntryRecord, Model, Mutation},
    persistence::state::UiStateStore,
    widgets::table::{self, table},
};
use barnacle_l10n::Translations;
use iced::{
    Element, Length, Point, Task,
    widget::{checkbox, mouse_area, pin, stack, text},
};

use crate::ui::tabs::mods::context_menu::ContextMenu;

pub mod add_mod_menu;
pub mod context_menu;

#[derive(Debug, Clone)]
pub enum Message {
    ModEntryToggled {
        record: ModEntryRecord,
        enabled: bool,
    },
    ModEntryDeleted(ModEntryRecord),
    ShowContextMenu {
        record: ModEntryRecord,
        position: Point,
    },
    ModEntryMoved {
        record: ModEntryRecord,
        new_position: u32,
    },
    ClickedOutsideContextMenu,
    ContextMenu(context_menu::Message),
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
    context_menu: Option<ContextMenu>,
}

impl ModsTab {
    pub fn new(ui_state: UiStateStore) -> Self {
        Self {
            ui_state,
            context_menu: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ModEntryDeleted(entry) => {
                Action::Mutate(Mutation::DeleteModEntry(entry.object()))
            }
            Message::ModEntryToggled {
                record: entry_record,
                enabled,
            } => Action::Mutate(Mutation::SetModEntryEnabled {
                entry: entry_record.object(),
                enabled,
            }),
            Message::ShowContextMenu { record, position } => {
                self.context_menu = Some(ContextMenu::new(position, record));
                Action::None
            }
            Message::ModEntryMoved {
                record,
                new_position,
            } => Action::Mutate(Mutation::MoveModEntry {
                entry: record.object(),
                new_position,
            }),
            Message::ClickedOutsideContextMenu => {
                self.context_menu = None;
                Action::None
            }
            Message::ContextMenu(message) => {
                if let Some(context_menu) = &self.context_menu {
                    match context_menu.update(message) {
                        context_menu::Action::Mutate(mutation) => {
                            self.context_menu = None;

                            Action::Mutate(mutation)
                        }
                    }
                } else {
                    Action::None
                }
            }
        }
    }

    pub fn view<'a>(&'a self, model: &'a Model, t: &Translations) -> Element<'a, Message> {
        let columns = [
            table::column(text("Name"), |entry_record: ModEntryRecord| {
                text(entry_record.name.clone())
            }),
            table::column(text("Status"), |entry_record: ModEntryRecord| {
                checkbox(entry_record.enabled).on_toggle(move |state| Message::ModEntryToggled {
                    record: entry_record.clone(),
                    enabled: state,
                })
            }),
        ];

        let table_view = table(columns, model.mod_entries())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_right_click_row(|index, cursor_pos| Message::ShowContextMenu {
                record: model.mod_entries().get(index).unwrap().clone(),
                position: cursor_pos,
            })
            .on_row_moved(|movement| Message::ModEntryMoved {
                record: model.mod_entries().get(movement.index).unwrap().clone(),
                new_position: movement.new_position.try_into().unwrap(),
            });

        if let Some(context_menu) = &self.context_menu {
            let context_menu =
                pin(context_menu.view(t).map(Message::ContextMenu)).position(context_menu.position);

            stack![
                table_view,
                mouse_area(context_menu).on_press(Message::ClickedOutsideContextMenu)
            ]
            .height(Length::Fill)
            .into()
        } else {
            table_view.into()
        }
    }
}
