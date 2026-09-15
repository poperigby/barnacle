use crate::{Repository, profile, repository::DeployKind};

#[tokio::test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile = game.add_profile("Test").await.unwrap();

    assert!(profile.dir().await.unwrap().exists());
}

#[tokio::test]
async fn test_add_duplicate() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    game.add_profile("Test").await.unwrap();

    assert!(matches!(
        game.add_profile("Test").await,
        Err(profile::AddError::DuplicateName { .. })
    ))
}

#[tokio::test]
async fn test_remove() {
    let repo = Repository::in_memory().await;
    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let _mod = game.new_mod("test_mod").empty().await;

    let profile = game.add_profile("Test").await.unwrap();
    let mod_entry = profile.add_mod_entry(_mod).await.unwrap();

    assert_eq!(game.profiles().await.unwrap().len(), 1);

    let dir = profile.dir().await.unwrap();

    profile.remove().await.unwrap();

    // Check the child mod entries were also removed
    assert!(mod_entry.remove().await.is_err());

    assert!(!dir.exists());
    assert_eq!(game.profiles().await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_list() {
    let repo = Repository::in_memory().await;
    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();

    assert_eq!(game.profiles().await.unwrap().len(), 0);

    game.add_profile("Cool Profile").await.unwrap();

    assert_eq!(repo.games().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_parent() {
    let repo = Repository::in_memory().await;

    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let profile = game.add_profile("Test").await.unwrap();

    assert_eq!(profile.parent().await.unwrap(), game);
}

#[tokio::test]
async fn test_activate() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    let profile1 = game.add_profile("Test1").await.unwrap();
    let profile2 = game.add_profile("Test2").await.unwrap();

    // First profile should have been automatically set as active
    assert!(profile1.is_active().await.unwrap());

    profile2.activate().await.unwrap();

    assert!(profile2.is_active().await.unwrap());
}

#[tokio::test]
async fn test_dir() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap();

    let profile = game.add_profile("Test").await.unwrap();

    let expected_dir = repo
        .cfg
        .read()
        .library_dir()
        .join(game.id().to_string())
        .join("profiles")
        .join(profile.id.to_string());

    assert_eq!(profile.dir().await.unwrap(), expected_dir);
}

#[tokio::test]
async fn test_remove_made_next_profile_active() {
    let repo = Repository::in_memory().await;
    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();

    let profile1 = game.add_profile("Test1").await.unwrap();
    let profile2 = game.add_profile("Test2").await.unwrap();

    profile1.activate().await.unwrap();
    assert!(profile1.is_active().await.unwrap());

    profile1.remove().await.unwrap();
    assert!(profile2.is_active().await.unwrap());
}

#[tokio::test]
async fn test_switching_games_preserves_each_games_active_profile() {
    let repo = Repository::in_memory().await;

    let game1 = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    game1.activate().await.unwrap();

    let profile1 = game1.add_profile("Test1").await.unwrap();
    profile1.activate().await.unwrap();
    game1.add_profile("Test2").await.unwrap();

    let game2 = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile2 = game2.add_profile("Test2").await.unwrap();

    game2.activate().await.unwrap();

    assert!(profile2.is_active().await.unwrap());

    game1.activate().await.unwrap();

    assert!(profile1.is_active().await.unwrap());
}
