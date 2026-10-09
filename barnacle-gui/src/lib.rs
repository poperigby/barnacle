pub mod icons;
pub mod log;
pub mod model;
pub mod persistence;
pub mod widgets;

use barnacle_l10n::Translation;
use iced::widget::{
    Text,
    text::{self, Catalog},
};

pub trait TranslationElementExt {
    fn element<'a, Theme, Renderer>(self) -> Text<'a, Theme, Renderer>
    where
        Theme: 'a + Catalog,
        Renderer: 'a;
}

impl TranslationElementExt for Translation {
    fn element<'a, AppMessage>(self) -> Text<'a, AppMessage>
    where
        AppMessage: 'a,
    {
        text(self.to_string()).into()
    }
}
