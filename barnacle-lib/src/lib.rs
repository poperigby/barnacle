pub mod fs;
pub mod repository;

pub use repository::{DeployKind, Game, Mod, ModEntry, Profile, Repository, Tool};

pub mod error {
    pub use crate::repository::handles::error::{GetFieldError, LoadModelError, ModelKind};
}

pub mod game {
    pub use crate::repository::handles::game::*;
}

pub mod profile {
    pub use crate::repository::handles::profile::*;
}

pub mod mod_ {
    pub use crate::repository::handles::mod_::*;
}

pub mod mod_entry {
    pub use crate::repository::handles::mod_entry::*;
}

pub mod tool {
    pub use crate::repository::handles::tool::*;
}
