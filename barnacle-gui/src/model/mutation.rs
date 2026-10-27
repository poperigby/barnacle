use std::{path::PathBuf, sync::Arc};

use barnacle_lib::{
    Repository,
    games::Kind,
    mods::builder::FileProgress,
    repository::{
        Game, Mod, ModEntry, Profile,
        item_entries::{ItemEntry, ItemGroup},
    },
};
use futures::executor::block_on;
use iced::{Task, task::sipper};

pub enum Event<Message> {
    Progress(Message),
    Done(Done<Message>),
}

type Error = Arc<anyhow::Error>;
#[derive(Debug, Clone)]
pub struct Done<Message> {
    /// The result of running the [`Work`]. This will let the App know if the [`Mutation`] succeeded or
    /// not. If it did, the App can reload the model.
    pub result: Result<(), Error>,
    /// An optional message to let the original child component know that the [`Mutation`]
    /// succeeded.
    pub notify: Option<Message>,
}

#[must_use]
pub struct Mutation<Message> {
    work: Box<dyn FnOnce(Repository) -> Task<Event<Message>>>,
}

impl<Message> Mutation<Message>
where
    Message: Send + 'static,
{
    pub fn run(self, repo: &Repository) -> Task<Event<Message>> {
        (self.work)(repo.clone())
    }

    pub fn map<Other>(self, f: impl Fn(Message) -> Other + Send + 'static) -> Mutation<Other>
    where
        Other: Send + 'static,
    {
        Mutation {
            work: Box::new(move |repo| {
                (self.work)(repo).map(move |event| match event {
                    Event::Progress(m) => Event::Progress(f(m)),
                    Event::Done(d) => Event::Done(Done {
                        result: d.result,
                        notify: d.notify.map(&f),
                    }),
                })
            }),
        }
    }

    fn perform<F, Fut>(f: F) -> Self
    where
        F: FnOnce(Repository) -> Fut + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        Self {
            work: Box::new(move |repo| {
                Task::perform(f(repo), |result| {
                    Event::Done(Done {
                        result: result.map_err(Arc::new),
                        notify: None,
                    })
                })
            }),
        }
    }

    // Games

    pub fn create_game(name: String, kind: Kind) -> Self {
        Self::perform(async move |repo| {
            let game = repo.add_game(&name, kind).await?;
            game.add_profile("Default").await?;

            Ok(())
        })
    }

    pub fn delete_game(game: Game) -> Self {
        Self::perform(async move |_| {
            game.remove().await?;
            Ok(())
        })
    }

    pub fn activate_game(game: Game) -> Self {
        Self::perform(async move |_| {
            game.activate().await?;
            Ok(())
        })
    }

    // Profiles

    pub fn create_profile(game: Game, name: String) -> Self {
        Self::perform(async move |_| {
            game.add_profile(&name).await?;
            Ok(())
        })
    }

    pub fn delete_profile(profile: Profile) -> Self {
        Self::perform(async move |_| {
            profile.remove().await?;
            Ok(())
        })
    }

    pub fn activate_profile(profile: Profile) -> Self {
        Self::perform(async move |_| {
            profile.activate().await?;
            Ok(())
        })
    }

    pub fn edit_profile(profile: Profile, new_name: String) -> Self {
        Self::perform(async move |_| {
            profile.set_name(&new_name).await?;
            Ok(())
        })
    }

    // Mods

    pub fn add_empty_mod(name: String) -> Self {
        Self::perform(async move |repo| {
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
        on_progress: impl Fn(FileProgress) -> Message + Send + 'static,
    ) -> Self {
        Self {
            work: Box::new(move |repo| {
                Task::sip(
                    sipper(async move |mut progress| {
                        let report = Box::new(move |file_progress| {
                            block_on(progress.send(file_progress));
                        });

                        if let Some(game) = repo.active_game().await? {
                            let mod_ = game.new_mod(&name).from_dir(&path, Some(report)).await;

                            if let Some(profile) = game.active_profile().await? {
                                profile.add_mod_entry(mod_).await?;
                            }
                        }

                        anyhow::Ok(())
                    }),
                    move |progress| Event::Progress(on_progress(progress)),
                    move |result| {
                        Event::Done(Done {
                            result: result.map_err(Arc::new),
                            notify: None,
                        })
                    },
                )
            }),
        }
    }

    pub fn add_mod_from_archive(
        name: String,
        path: PathBuf,
        on_progress: impl Fn(FileProgress) -> Message + Send + 'static,
    ) -> Self {
        Self {
            work: Box::new(move |repo| {
                Task::sip(
                    sipper(async move |mut progress| {
                        let report = Box::new(move |file_progress| {
                            block_on(progress.send(file_progress));
                        });

                        if let Some(game) = repo.active_game().await? {
                            let mod_ = game
                                .new_mod(&name)
                                .from_archive(&path)
                                .install(Some(report))
                                .await;

                            if let Some(profile) = game.active_profile().await? {
                                profile.add_mod_entry(mod_).await?;
                            }
                        }

                        anyhow::Ok(())
                    }),
                    move |progress| Event::Progress(on_progress(progress)),
                    move |result| {
                        Event::Done(Done {
                            result: result.map_err(Arc::new),
                            notify: None,
                        })
                    },
                )
            }),
        }
    }

    pub fn delete_mod(mod_: Mod) -> Self {
        Self::perform(async move |_| {
            mod_.remove().await?;

            Ok(())
        })
    }

    // Mod entries

    pub fn set_mod_entry_enabled(entry: ModEntry, enabled: bool) -> Self {
        Self::perform(async move |_| {
            entry.set_enabled(enabled).await?;

            Ok(())
        })
    }

    pub fn move_mod_entry(entry: ModEntry, new_position: u32) -> Self {
        Self::perform(async move |_| {
            entry.set_position(new_position).await?;

            Ok(())
        })
    }

    pub fn delete_mod_entry(entry: ModEntry) -> Self {
        Self::perform(async move |_| {
            entry.remove().await?;

            Ok(())
        })
    }

    // Items

    pub fn set_item_entry_enabled(entry: ItemEntry, enabled: bool) -> Self {
        Self::perform(async move |_| {
            entry.set_enabled(enabled).await;

            Ok(())
        })
    }

    pub fn auto_sort_item_group(group: ItemGroup) -> Self {
        Self::perform(async move |_| {
            group.auto_sort().await?;

            Ok(())
        })
    }
}
