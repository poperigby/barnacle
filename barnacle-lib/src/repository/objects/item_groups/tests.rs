use tokio::test;

use crate::{DeployKind, Repository};

#[test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    // This will add the correct ItemGroups based on the DeployKind
    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    let mut keys = Vec::new();
    for group in game.item_groups().await {
        keys.push(group.key().await);
    }

    assert!(keys.contains(&"plugin".to_string()));
    assert!(keys.contains(&"archive".to_string()));
}
