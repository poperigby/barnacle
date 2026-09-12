//! Core domain entities for Barnacle.
//!
//! These types represent games, profiles, mods, and other elements managed by
//! the system. They provide a unified interface for inspecting and mutating
//! these elements, handling all necessary operations behind the scenes.

pub(crate) mod error;
pub mod game;
pub mod mod_;
pub mod mod_entry;
pub mod profile;
pub mod target;
pub mod tool;

pub use game::Game;
pub use mod_::Mod;
pub use mod_entry::ModEntry;
pub use profile::Profile;
pub use target::Target;
pub use tool::Tool;
