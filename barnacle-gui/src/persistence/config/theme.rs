use deunicode::deunicode;
use heck::ToUpperCamelCase;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Debug, Clone, PartialEq)]
pub struct Theme(iced::Theme);

impl Default for Theme {
    fn default() -> Self {
        Self(iced::Theme::Dark)
    }
}

impl From<&Theme> for iced::Theme {
    fn from(theme: &Theme) -> Self {
        theme.0.clone()
    }
}

impl From<iced::Theme> for Theme {
    fn from(value: iced::Theme) -> Self {
        Self(value)
    }
}

impl Serialize for Theme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&key(&self.0))
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        iced::Theme::ALL
            .iter()
            .find(|theme| key(theme) == value)
            .cloned()
            .map(Self)
            .ok_or_else(|| de::Error::custom(format!("unknown theme `{value}`")))
    }
}

fn key(theme: &iced::Theme) -> String {
    let key = theme.to_string().to_upper_camel_case();

    // The string representations can have Unicode characters, such as diacritics. We'll strip those.
    deunicode(&key)
}
