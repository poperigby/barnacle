use std::path::PathBuf;

use barnacle_lib::{
    Repository,
    repository::{DeployKind, Game, Mod, ModEntry, Profile},
};

#[derive(Debug)]
pub enum Mutation {
    // Games
    CreateGame {
        name: String,
        deploy_kind: DeployKind,
    },
    DeleteGame(Game),
    ActivateGame(Game),

    // Profiles
    CreateProfile {
        game: Game,
        name: String,
    },
    DeleteProfile(Profile),
    ActivateProfile(Profile),
    EditProfile {
        profile: Profile,
        new_name: String,
    },

    // Mods
    AddEmptyMod {
        name: String,
    },
    AddModFromDir {
        name: String,
        path: PathBuf,
    },
    AddModFromArchive {
        name: String,
        path: PathBuf,
    },
    DeleteMod(Mod),

    // Mod entries
    SetModEntryEnabled {
        entry: ModEntry,
        enabled: bool,
    },
    DeleteModEntry(ModEntry),
}

impl Mutation {
    pub async fn run(self, repo: &Repository) {
        use Mutation::*;

        match self {
            // Games
            CreateGame { name, deploy_kind } => {
                let game = repo.add_game(&name, deploy_kind).await.unwrap();
                game.add_profile("Default").await.unwrap();
            }
            DeleteGame(game) => {
                game.remove().await.unwrap();
            }
            ActivateGame(game) => {
                game.activate().await.unwrap();
            }

            // Profiles
            CreateProfile { game, name } => {
                game.add_profile(&name).await.unwrap();
            }
            DeleteProfile(profile) => {
                profile.remove().await.unwrap();
            }
            ActivateProfile(profile) => {
                profile.activate().await.unwrap();
            }
            EditProfile { profile, new_name } => {
                profile.set_name(&new_name).await.unwrap();
            }

            // Mods
            AddEmptyMod { name } => {
                let active_game = repo.active_game().await.unwrap().unwrap();

                let mod_ = active_game.new_mod(&name).empty().await;

                if let Some(active_profile) = active_game.active_profile().await.unwrap() {
                    active_profile.add_mod_entry(mod_).await.unwrap();
                }
            }
            AddModFromDir { name, path } => {
                let active_game = repo.active_game().await.unwrap().unwrap();

                let mod_ = active_game
                    .new_mod(&name)
                    .import_dir(&path, None)
                    .await;

                if let Some(active_profile) = active_game.active_profile().await.unwrap() {
                    active_profile.add_mod_entry(mod_).await.unwrap();
                }
            }
            AddModFromArchive { name, path } => {
                let active_game = repo.active_game().await.unwrap().unwrap();

                let mod_ = active_game
                    .new_mod(&name)
                    .import_archive(&path)
                    .with_root(&PathBuf::from("PerksOfMorrowind"))
                    .intall(None)
                    .await;

                if let Some(active_profile) = active_game.active_profile().await.unwrap() {
                    active_profile.add_mod_entry(mod_).await.unwrap();
                }
            }
            DeleteMod(mod_) => {
                mod_.remove().await.unwrap();
            }

            // Mod entries
            SetModEntryEnabled { entry, enabled } => {
                entry.set_enabled(enabled).await.unwrap();
            }
            DeleteModEntry(entry) => {
                entry.remove().await.unwrap();
            }
        }
    }
}
