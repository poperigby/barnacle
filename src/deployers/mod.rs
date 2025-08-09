use crate::data::v1::{games::Game, profiles::Profile};

pub mod overlay;

/// Deploy or undeploy a profile to a target game.
pub trait Deploy {
    /// Construct a deployer for a given profile and game, performing setup as needed.
    fn setup(game: &Game, profile: &Profile) -> Self;
    fn deploy(&mut self);
    fn undeploy(&mut self);
}
