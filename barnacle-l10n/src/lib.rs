use std::env;

use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use icu::locale::LanguageIdentifier;
use sys_locale::get_locales;
use thiserror::Error;

pub use fluent_static::MessageBundle as TranslationBundle;

include!(concat!(env!("OUT_DIR"), "/messages.rs"));

/// Returns a [`Translations`] bundle, which has methods that return localized strings. If a `requested_lang`
/// is provided, it will attempt to find translations for that language, otherwise it will fall back to system
/// languages in order of priority.
pub fn translations(
    requested_lang: Option<LanguageIdentifier>,
) -> Result<Translations, TranslationsError> {
    let requested: Vec<LanguageIdentifier> = requested_lang
        .into_iter()
        .chain(get_locales().filter_map(|tag| tag.parse().ok()))
        .collect();

    let available: Vec<LanguageIdentifier> = Translations::supported_language_ids()
        .iter()
        .map(|id| id.parse().map_err(TranslationsError::ParseLanguageId))
        .collect::<Result<_, _>>()?;

    let default: LanguageIdentifier = "en-US"
        .parse()
        .expect("en-US is a valid LanguageIdentifier");

    let supported = negotiate_languages(
        &requested,
        &available,
        Some(&default),
        NegotiationStrategy::Lookup,
    );

    Ok(Translations::get(
        &supported
            .first()
            .expect("supported must contain at least 1 language")
            .to_string(),
    )
    .expect("this will always produce a valid bundle, as we have en-US as a fallback"))
}

#[derive(Debug, Clone, Error)]
pub enum TranslationsError {
    #[error("Failed to parse language ID from Translations into a LanguageIdentifier: {0}")]
    ParseLanguageId(icu::locale::ParseError),
}

#[cfg(test)]
mod test {
    use fluent_static::LanguageAware;

    use crate::translations;

    #[test]
    fn test_system_language() {
        let translations = translations(None).unwrap();
        let lang = translations.language.language_id();

        assert_eq!(lang, "en-US");
    }

    #[test]
    fn test_system_language_with_requested_lang() {
        let translations = translations(Some("es".parse().unwrap())).unwrap();
        let lang = translations.language.language_id();

        assert_eq!(lang, "es");
    }
}
