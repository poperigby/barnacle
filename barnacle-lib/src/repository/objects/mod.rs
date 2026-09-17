//! Core domain objects
//!
//! These types represent games, profiles, mods, and other elements managed by
//! the system. They provide a unified interface for inspecting and mutating
//! these elements, handling all necessary operations behind the scenes.

pub(crate) mod error;
pub mod games;
pub mod item_entries;
pub mod item_lists;
pub mod items;
pub mod mod_entries;
pub mod mods;
pub mod profiles;
pub mod targets;
pub mod tools;

pub use games::Game;
pub use item_entries::ItemEntry;
pub use mod_entries::ModEntry;
pub use mods::Mod;
pub use profiles::Profile;
pub use targets::Target;
pub use tools::Tool;
