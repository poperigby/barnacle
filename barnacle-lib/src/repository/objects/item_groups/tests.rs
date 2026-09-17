use tokio::test;

use crate::{DeployKind, Repository, repository::item_groups::find_by_key};

#[test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    // This will add the correct ItemGroups based on the DeployKind
    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();

    find_by_key(repo.db.conn(), game.id(), "plugin").await;
    find_by_key(repo.db.conn(), game.id(), "archive").await;
}
