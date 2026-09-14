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
    AddMod {
        name: String,
        path: Option<PathBuf>,
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
            AddMod { name, path } => {
                let game = match repo.active_game().await.unwrap() {
                    Some(game) => game,
                    None => return,
                };

                let mod_ = if let Some(path) = path {
                    game.new_mod(&name)
                        .unwrap()
                        .import_dir(&path, |p| println!("{}", p.bytes_total))
                        .await
                } else {
                    game.new_mod(&name).unwrap().empty().await
                };

                if let Some(profile) = game.active_profile().await.unwrap() {
                    profile.add_mod_entry(mod_).await.unwrap();
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
