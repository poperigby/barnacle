use std::{path::PathBuf, sync::Arc};

use barnacle_lib::{
    Repository,
    games::Kind,
    mods::builder::{FileProgress, ProgressCallback},
    repository::{
        Game, Mod, ModEntry, Profile,
        item_entries::{ItemEntry, ItemGroup},
    },
};
use futures::executor::block_on;
use iced::{Task, task::sipper};
use sipper::Sender;

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

    pub fn perform<F>(work: impl FnOnce(Repository) -> F + Send + 'static) -> Self
    where
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        Self {
            work: Box::new(move |repo| {
                Task::perform(work(repo), |result| {
                    Event::Done(Done {
                        result: result.map_err(Arc::new),
                        notify: None,
                    })
                })
            }),
        }
    }

    pub fn sip<F>(
        work: impl FnOnce(Repository, ProgressCallback) -> F + Send + 'static,
        on_progress: impl Fn(FileProgress) -> Message + Send + 'static,
        on_done: impl Fn() -> Message + Send + 'static,
    ) -> Self
    where
        F: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        Self {
            work: Box::new(move |repo| {
                Task::sip(
                    sipper(async move |mut sender| {
                        let report: ProgressCallback = Box::new(move |p| block_on(sender.send(p)));

                        work(repo, report).await
                    }),
                    move |progress| Event::Progress(on_progress(progress)),
                    move |result| {
                        Event::Done(Done {
                            result: result.map_err(Arc::new),
                            notify: Some(on_done()),
                        })
                    },
                )
            }),
        }
    }
}
