pub mod icons;
pub mod log;
pub mod model;
pub mod persistence;
pub mod widgets;

use barnacle_l10n::Translation;
use iced::widget::{Text, text};

pub trait TranslationElementExt<'a> {
    fn element(self) -> Text<'a>;
}

impl<'a> TranslationElementExt<'a> for Translation {
    fn element(self) -> Text<'a> {
        text(self.to_string())
    }
}
