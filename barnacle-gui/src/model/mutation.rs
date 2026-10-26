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
use futures::future::BoxFuture;
use iced::Task;

type Work = Box<dyn FnOnce(Repository) -> BoxFuture<'static, anyhow::Result<()>>>;

#[must_use]
pub struct Mutation {
    work: Work,
}

impl Mutation {
    fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce(Repository) -> Fut + 'static,
        Fut: Future<Output = anyhow::Result<()>> + 'static + Send,
    {
        let work: Work = Box::new(|repo: Repository| Box::pin(f(repo)));

        Self { work }
    }

    pub fn run<F, Message>(self, repo: &Repository, on_completion: F) -> Task<Message>
    where
        F: FnOnce(anyhow::Result<()>) -> Message + Send + 'static,
        Message: Send + 'static,
    {
        Task::perform((self.work)(repo.clone()), on_completion)
    }

    // Games

    pub fn create_game(name: String, kind: Kind) -> Self {
        Self::new(async move |repo: Repository| {
            let game = repo.add_game(&name, kind).await?;
            game.add_profile("Default").await?;

            Ok(())
        })
    }

    pub fn delete_game(game: Game) -> Self {
        Self::new(move |_repo: Repository| async move {
            game.remove().await?;
            Ok(())
        })
    }

    pub fn activate_game(game: Game) -> Self {
        Self::new(move |_repo: Repository| async move {
            game.activate().await?;
            Ok(())
        })
    }

    // Profiles

    pub fn create_profile(game: Game, name: String) -> Self {
        Self::new(move |_repo: Repository| async move {
            game.add_profile(&name).await?;
            Ok(())
        })
    }

    pub fn delete_profile(profile: Profile) -> Self {
        Self::new(move |_repo: Repository| async move {
            profile.remove().await?;
            Ok(())
        })
    }

    pub fn activate_profile(profile: Profile) -> Self {
        Self::new(move |_repo: Repository| async move {
            profile.activate().await?;
            Ok(())
        })
    }

    pub fn edit_profile(profile: Profile, new_name: String) -> Self {
        Self::new(move |_repo: Repository| async move {
            profile.set_name(&new_name).await?;
            Ok(())
        })
    }

    // Mods

    pub fn add_empty_mod(name: String) -> Self {
        Self::new(move |repo: Repository| async move {
            if let Some(game) = repo.active_game().await? {
                let mod_ = game.new_mod(&name).empty().await;

                if let Some(profile) = game.active_profile().await? {
                    profile.add_mod_entry(mod_).await?;
                }
            }

            Ok(())
        })
    }

    pub fn add_mod_from_dir(
        name: String,
        path: PathBuf,
        progress: Option<ProgressCallback>,
    ) -> Self {
        Self::new(move |repo: Repository| async move {
            if let Some(game) = repo.active_game().await? {
                let mod_ = game.new_mod(&name).from_dir(&path, progress).await;

                if let Some(profile) = game.active_profile().await? {
                    profile.add_mod_entry(mod_).await?;
                }
            }

            Ok(())
        })
    }

    pub fn add_mod_from_archive(
        name: String,
        path: PathBuf,
        progress: Option<ProgressCallback>,
    ) -> Self {
        Self::new(move |repo: Repository| async move {
            if let Some(game) = repo.active_game().await? {
                let mod_ = game
                    .new_mod(&name)
                    .from_archive(&path)
                    .with_root(&PathBuf::from("PerksOfMorrowind"))
                    .install(progress)
                    .await;

                if let Some(profile) = game.active_profile().await? {
                    profile.add_mod_entry(mod_).await?;
                }
            }

            Ok(())
        })
    }

    pub fn delete_mod(mod_: Mod) -> Self {
        Self::new(move |_repo: Repository| async move {
            mod_.remove().await?;
            Ok(())
        })
    }

    // Mod entries

    pub fn set_mod_entry_enabled(entry: ModEntry, enabled: bool) -> Self {
        Self::new(move |_repo: Repository| async move {
            entry.set_enabled(enabled).await?;
            Ok(())
        })
    }

    pub fn move_mod_entry(entry: ModEntry, new_position: u32) -> Self {
        Self::new(move |_repo: Repository| async move {
            entry.set_position(new_position).await.unwrap();
            Ok(())
        })
    }

    pub fn delete_mod_entry(entry: ModEntry) -> Self {
        Self::new(move |_repo: Repository| async move {
            entry.remove().await?;
            Ok(())
        })
    }

    // Items

    pub fn set_item_entry_enabled(entry: ItemEntry, enabled: bool) -> Self {
        Self::new(move |_repo: Repository| async move {
            entry.set_enabled(enabled).await;
            Ok(())
        })
    }

    pub fn auto_sort_item_group(group: ItemGroup) -> Self {
        Self::new(move |_repo: Repository| async move {
            group.auto_sort().await?;
            Ok(())
        })
    }
}
