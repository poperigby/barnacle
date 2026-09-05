//! Entrypoint of the application. This is reponsible for loading the application data and passing
//! it to the UI that lives in [`Workspace`].

use std::sync::Arc;

use barnacle_lib::Repository;
use fluent_i18n::i18n;
use iced::{
    Color, Element,
    Length::{self},
    Task, Theme, application,
    widget::{center, container, mouse_area, opaque, stack, text},
    window::Settings,
};
use parking_lot::RwLock;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::{config::GuiConfig, data::AppData, workspace::Workspace};

pub mod config;
pub mod data;
pub mod icons;
pub mod workspace;

i18n!("locales", fallback = "en-US");

fn main() -> iced::Result {
    human_panic::setup_panic!();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut settings = Settings::default();
    settings.platform_specific.application_id = App::TITLE.to_string();

    application(App::new, App::update, App::view)
        .theme(App::theme)
        .title(App::title)
        .window(settings)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Initialized { repo: Repository, data: AppData },
    IntializeFailed(String),

    Refreshed(AppData),
    RefreshFailed(String),

    Workspace(workspace::Message),
}

#[derive(Debug, Clone)]
enum State {
    Loading,
    Error(String),
    Ready {
        repo: Repository,
        data: AppData,
        workspace: Workspace,
    },
}

pub struct App {
    state: State,
    cfg: Arc<RwLock<GuiConfig>>,

    title: String,
    theme: Theme,
}

impl App {
    const TITLE: &str = "Barnacle";

    fn new() -> (Self, Task<Message>) {
        let state = State::Loading;
        let cfg = Arc::new(RwLock::new(GuiConfig::load()));
        let theme = cfg.read().theme();

        (
            Self {
                state: state.clone(),
                cfg: cfg.clone(),

                title: Self::TITLE.to_string(),
                theme,
            },
            Self::load(),
        )
    }

    fn load() -> Task<Message> {
        Task::perform(
            {
                async {
                    let repo = Repository::new().await;
                    let data = AppData::load(&repo).await;
                    (repo, data)
                }
            },
            |(repo, data)| Message::Initialized { repo, data },
        )
    }

    fn refresh(repo: Repository) -> Task<Message> {
        Task::perform(
            async move {
                let data = AppData::load(&repo).await;
                data
            },
            Message::Refreshed,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized { data, repo } => {
                let (workspace, workspace_task) = Workspace::init(&repo, &data, self.cfg.clone());

                self.state = State::Ready {
                    repo,
                    data,
                    workspace,
                };

                workspace_task.map(Message::Workspace)
            }
            Message::IntializeFailed(e) => {
                self.state = State::Error(e);
                Task::none()
            }
            Message::Refreshed(new_data) => {
                let State::Ready {
                    data: current_data,
                    workspace,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let task = workspace.sync(&new_data);
                *current_data = new_data;

                task.map(Message::Workspace)
            }
            Message::RefreshFailed(e) => {
                self.state = State::Error(e);
                Task::none()
            }
            Message::Workspace(message) => match &mut self.state {
                State::Ready {
                    repo, workspace, ..
                } => match workspace.update(&repo.clone(), message) {
                    workspace::Action::None => Task::none(),
                    workspace::Action::Run(task) => task.map(Message::Workspace),
                    workspace::Action::Refresh => Self::refresh(repo.clone()),
                },
                _ => panic!("FUCK"),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => text!("LOADING").into(),
            State::Error(_) => panic!("ERROR"),
            State::Ready { workspace, .. } => workspace.view().map(Message::Workspace),
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_click_outside: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mouse_area = mouse_area(center(opaque(content)).style(|_theme| {
        container::Style {
            background: Some(
                Color {
                    a: 0.8,
                    ..Color::BLACK
                }
                .into(),
            ),
            ..container::Style::default()
        }
    }));

    stack![
        base.into(),
        opaque(if let Some(msg) = on_click_outside {
            mouse_area.on_press(msg)
        } else {
            mouse_area
        })
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
