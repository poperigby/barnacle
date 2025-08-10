use crate::data::v1::{games::Game, profiles::Profile};

pub mod overlay;

/// Deploy or undeploy a profile to a target game.
pub trait Deploy {
    fn deploy(&mut self);
    fn undeploy(&mut self);
}
