use std::assert_matches;

use crate::Repository;

use super::*;

#[tokio::test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    let game1 = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    repo.add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    let games = repo.games().await.unwrap();

    assert!(game1.dir().await.unwrap().exists());
    assert_eq!(games.len(), 2);
    assert_eq!(games.first().unwrap().name().await.unwrap(), "Morrowind");
    assert_eq!(
        games.last().unwrap().deploy_kind().await.unwrap(),
        DeployKind::Skyrim
    );
}

#[tokio::test]
async fn test_add_duplicate() {
    let repo = Repository::in_memory().await;

    let _game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    assert_matches!(
        repo.add_game("Morrowind", DeployKind::OpenMW).await,
        Err(AddError::DuplicateName { .. }),
    )
}

#[tokio::test]
async fn test_remove() {
    let repo = Repository::in_memory().await;

    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let profile = game.add_profile("test_profile_1").await.unwrap();
    let mod_ = game.new_mod("test_mod").empty().await;

    assert_eq!(repo.games().await.unwrap().len(), 1);

    let dir = game.dir().await.unwrap();

    game.remove().await.unwrap();

    // Attempt to remove already removed profile and mod entries
    assert!(profile.remove().await.is_err());
    assert!(mod_.remove().await.is_err());

    assert!(!dir.exists());
    assert_eq!(repo.games().await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_remove_made_next_game_active() {
    let repo = Repository::in_memory().await;
    let game1 = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let game2 = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    game1.activate().await.unwrap();
    assert!(game1.is_active().await.unwrap());

    game1.remove().await.unwrap();
    assert!(game2.is_active().await.unwrap());
}

#[tokio::test]
async fn test_list() {
    let repo = Repository::in_memory().await;

    assert_eq!(repo.games().await.unwrap().len(), 0);

    repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();

    assert_eq!(repo.games().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_name() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap();

    game.name().await.unwrap();
}

#[tokio::test]
async fn test_set_name() {
    let repo = Repository::in_memory().await;

    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();

    assert_eq!(game.name().await.unwrap(), "Skyrim");

    game.set_name("Skyrim 3: Electric Boogaloo").await.unwrap();

    assert_eq!(game.name().await.unwrap(), "Skyrim 3: Electric Boogaloo");
}

#[tokio::test]
async fn test_deploy_kind() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap();

    game.deploy_kind().await.unwrap();
}

#[tokio::test]
async fn test_dir() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap();

    let expected_dir = repo.cfg.read().library_dir().join(game.id.to_string());

    assert_eq!(game.dir().await.unwrap(), expected_dir);
}

#[tokio::test]
async fn test_activate() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    game.activate().await.unwrap();

    assert!(game.is_active().await.unwrap());
    assert_eq!(repo.active_game().await.unwrap().unwrap(), game);
}
