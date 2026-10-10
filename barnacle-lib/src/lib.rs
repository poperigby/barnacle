pub mod fs;
pub mod integrations;
pub mod repository;

#[cfg(test)]
mod test;

use barnacle_l10n::Translations;
pub use repository::{Game, Mod, ModEntry, Profile, Repository, Tool};

use crate::repository::config::Cfg;

pub mod error {
    pub use crate::repository::objects::error::{GetFieldError, LoadModelError, ModelKind};
}

pub mod games {
    pub use crate::repository::objects::games::*;
}

pub mod profiles {
    pub use crate::repository::objects::profiles::*;
}

pub mod mods {
    pub use crate::repository::objects::mods::*;
}

pub mod mod_entries {
    pub use crate::repository::objects::mod_entries::*;
}

pub mod tools {
    pub use crate::repository::objects::tools::*;
}

pub(crate) async fn translations(
    cfg: &Cfg,
) -> Result<Translations, barnacle_l10n::TranslationsError> {
    let cfg_locale = cfg.read().locale().map(|l| l.id.clone());
    barnacle_l10n::translations(cfg_locale)
}
