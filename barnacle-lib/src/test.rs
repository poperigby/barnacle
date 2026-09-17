use rstest::fixture;

use crate::{DeployKind, Game, Profile, Repository};

#[fixture]
pub async fn game() -> Game {
    let repo = Repository::new().await;

    repo.add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap()
}

#[fixture]
pub async fn profile() -> Profile {
    let repo = Repository::new().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    game.add_profile("ULTRA GRAPHICS EDITION 2028!!!")
        .await
        .unwrap()
}
