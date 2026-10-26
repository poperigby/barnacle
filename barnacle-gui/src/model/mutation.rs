use std::path::PathBuf;

use barnacle_lib::{
    Repository,
    games::Kind,
    mods::builder::ProgressCallback,
    repository::{
        Game, Mod, ModEntry, Profile,
        item_entries::{ItemEntry, ItemGroup},
    },
};

#[derive(Debug)]
pub enum Mutation {
    // Games
    CreateGame { name: String, deploy_kind: Kind },
    DeleteGame(Game),
    ActivateGame(Game),

    // Profiles
    CreateProfile { game: Game, name: String },
    DeleteProfile(Profile),
    ActivateProfile(Profile),
    EditProfile { profile: Profile, new_name: String },

    // Mods
    AddEmptyMod { name: String },
    AddModFromDir { name: String, path: PathBuf },
    AddModFromArchive { name: String, path: PathBuf },
    DeleteMod(Mod),

    // Mod entries
    SetModEntryEnabled { entry: ModEntry, enabled: bool },
    MoveModEntry { entry: ModEntry, new_position: u32 },
    DeleteModEntry(ModEntry),

    // Items
    SetItemEntryEnabled { entry: ItemEntry, enabled: bool },
    AutoSortItemGroup { group: ItemGroup },
}

impl Mutation {
    pub async fn run(self, repo: &Repository, progress: ProgressCallback) -> anyhow::Result<()> {
        use Mutation::*;

        match self {
            // Games
            CreateGame { name, deploy_kind } => {
                let game = repo.add_game(&name, deploy_kind).await?;
                game.add_profile("Default").await?;
            }
            DeleteGame(game) => {
                game.remove().await?;
            }
            ActivateGame(game) => {
                game.activate().await?;
            }

            // Profiles
            CreateProfile { game, name } => {
                game.add_profile(&name).await?;
            }
            DeleteProfile(profile) => {
                profile.remove().await?;
            }
            ActivateProfile(profile) => {
                profile.activate().await?;
            }
            EditProfile { profile, new_name } => {
                profile.set_name(&new_name).await?;
            }

            // Mods
            AddEmptyMod { name } => {
                if let Some(game) = repo.active_game().await? {
                    let mod_ = game.new_mod(&name).empty().await;

                    if let Some(profile) = game.active_profile().await? {
                        profile.add_mod_entry(mod_).await?;
                    }
                }
            }
            AddModFromDir { name, path } => {
                if let Some(game) = repo.active_game().await? {
                    let mod_ = game.new_mod(&name).from_dir(&path, Some(progress)).await;

                    if let Some(profile) = game.active_profile().await? {
                        profile.add_mod_entry(mod_).await?;
                    }
                }
            }
            AddModFromArchive { name, path } => {
                if let Some(game) = repo.active_game().await? {
                    let mod_ = game
                        .new_mod(&name)
                        .from_archive(&path)
                        .with_root(&PathBuf::from("PerksOfMorrowind"))
                        .install(Some(progress))
                        .await;

                    if let Some(profile) = game.active_profile().await? {
                        profile.add_mod_entry(mod_).await?;
                    }
                }
            }
            DeleteMod(mod_) => {
                mod_.remove().await?;
            }

            // Mod entries
            SetModEntryEnabled { entry, enabled } => {
                entry.set_enabled(enabled).await?;
            }
            MoveModEntry {
                entry,
                new_position,
            } => {
                entry.set_position(new_position).await.unwrap();
            }
            DeleteModEntry(entry) => {
                entry.remove().await?;
            }

            // Items
            SetItemEntryEnabled { entry, enabled } => {
                entry.set_enabled(enabled).await;
            }
            AutoSortItemGroup { group } => group.auto_sort().await?,
        }

        Ok(())
    }
}
