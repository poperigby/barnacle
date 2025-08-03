use crate::{games::Game, profiles::Profile};

pub mod overlay;

/// Deploy or undeploy a profile to a target game.
pub trait Deploy {
    type T: Deploy;

    /// Performs initialization of the deployer, including any structures or directories that it
    /// will require. This should be called when a profile is first created.
    fn init(game: &Game, profile: &Profile) -> Self::T;
    fn deploy(&mut self);
    fn undeploy(&mut self);
}
