use crate::{Repository, repository::DeployKind};

#[tokio::test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let mod_ = game.new_mod("Test").empty().await;

    assert!(mod_.dir().await.unwrap().exists());
}

#[tokio::test]
async fn test_add_duplicate() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    game.new_mod("Test").empty().await;

    // assert!(matches!(
    //     game.add_mod("Test").await.unwrap(),
    //     Err(mod_::AddError::DuplicateName { .. })
    // ))
}

#[tokio::test]
async fn test_remove() {
    let repo = Repository::in_memory().await;

    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let mod_ = game.new_mod("Test").empty().await;

    assert_eq!(game.mods().await.unwrap().len(), 1);

    let dir = mod_.dir().await.unwrap();

    mod_.remove().await.unwrap();

    assert_eq!(game.mods().await.unwrap().len(), 0);
    assert!(!dir.exists())
}

#[tokio::test]
async fn test_list() {
    let repo = Repository::in_memory().await;
    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();

    assert_eq!(game.mods().await.unwrap().len(), 0);

    game.new_mod("Better Spoon Textures 8K").empty().await;

    assert_eq!(game.mods().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_parent() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let mod_ = game.new_mod("Test").empty().await;

    assert_eq!(mod_.parent().await.unwrap(), game);
}

#[tokio::test]
async fn test_name() {
    let repo = Repository::in_memory().await;

    repo.add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap()
        .new_mod("Test")
        .empty()
        .await
        .name()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_dir() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Fallout: New Vegas", DeployKind::FalloutNV)
        .await
        .unwrap();

    let mod_ = game.new_mod("Test").empty().await;

    let expected_dir = repo
        .cfg
        .read()
        .library_dir()
        .join(game.id().to_string())
        .join("mods")
        .join(mod_.id.to_string());

    assert_eq!(mod_.dir().await.unwrap(), expected_dir);
}
