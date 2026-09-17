pub mod deployers;
pub mod fs;
pub mod repository;

pub use repository::{DeployKind, Game, Mod, ModEntry, Profile, Repository, Tool};

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
