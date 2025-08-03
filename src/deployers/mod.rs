use crate::{games::Game, profiles::Profile};

pub mod generic;

trait Deploy {
    type T: Deploy;

    fn setup(game: &Game, profile: &Profile) -> Self::T;
    fn deploy(&mut self);
    fn undeploy(&mut self);
}
