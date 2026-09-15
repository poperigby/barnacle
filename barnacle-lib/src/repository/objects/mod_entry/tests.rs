use super::*;
use crate::{Repository, repository::DeployKind};

#[tokio::test]
async fn test_add() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile = game.add_profile("Test").await.unwrap();

    let mod1 = game.new_mod("Super Duper Mod").empty().await;
    let mod2 = game.new_mod("Super Duper Mod: 2").empty().await;

    profile.add_mod_entry(mod1).await.unwrap();
    profile.add_mod_entry(mod2).await.unwrap();

    assert_eq!(profile.mod_entries().await.unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile = game.add_profile("Test").await.unwrap();

    let mut mod_entries = Vec::new();
    for i in 1..=6 {
        let m = game.new_mod(&format!("Mod{i}")).empty().await;
        mod_entries.push(profile.add_mod_entry(m).await.unwrap());
    }

    assert_eq!(profile.mod_entries().await.unwrap().len(), 6);

    async fn remove_and_check(entry: &ModEntry, profile: &Profile) {
        entry.clone().remove().await.unwrap();
        let entries = profile.mod_entries().await.unwrap();
        assert!(!entries.contains(entry));
    }

    remove_and_check(mod_entries.first().unwrap(), &profile).await; // first
    remove_and_check(mod_entries.get(3).unwrap(), &profile).await; // middle
    remove_and_check(mod_entries.get(5).unwrap(), &profile).await; // last

    // Check remaining entries are exactly the ones we expect
    let remaining: Vec<&ModEntry> = mod_entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match i {
            // Filter out the entries we removed
            0 | 3 | 5 => None,
            // These are the ones we expect to be here
            _ => Some(e),
        })
        .collect();
    assert_eq!(
        profile
            .mod_entries()
            .await
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        remaining
    );
}

#[tokio::test]
async fn test_parent() {
    let repo = Repository::in_memory().await;

    let game = repo.add_game("Skyrim", DeployKind::Skyrim).await.unwrap();
    let profile = game.add_profile("The Best Profile").await.unwrap();
    let mod_ = game
        .new_mod("Better Khajiit Balls 16K - Remastered - 2025 Edition - REAL")
        .empty()
        .await;
    let entry = profile.add_mod_entry(mod_).await.unwrap();

    assert_eq!(entry.parent().await.unwrap(), profile);
}

#[tokio::test]
async fn test_name() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile = game.add_profile("Test").await.unwrap();
    let mod_ = game.new_mod("Super Duper Mod").empty().await;

    profile
        .add_mod_entry(mod_)
        .await
        .unwrap()
        .name()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_enabled() {
    let repo = Repository::in_memory().await;

    let game = repo
        .add_game("Morrowind", DeployKind::OpenMW)
        .await
        .unwrap();
    let profile = game.add_profile("Test").await.unwrap();
    let mod_ = game.new_mod("Super Duper Mod").empty().await;

    let entry = profile.add_mod_entry(mod_).await.unwrap();

    assert!(entry.enabled().await.unwrap());

    entry.set_enabled(false).await.unwrap();

    assert!(!entry.enabled().await.unwrap());
}
