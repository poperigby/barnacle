use crate::Game;

pub mod openmw;

pub trait Deployer {
    fn deploy(game: &Game) -> impl Future<Output = ()> + Send;
}
