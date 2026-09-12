//! Entrypoint of the application. This is reponsible for loading the application data and passing
//! it to the main UI that lives in [`Ui`].

use barnacle_gui::{
    log,
    model::Model,
    persistence::{config::ConfigStore, state::UiStateStore},
};
use barnacle_lib::Repository;
use fluent_i18n::i18n;
use iced::{Element, Task, Theme, application, widget::text, window::Settings};
use tracing::error;

use crate::ui::Ui;

pub mod ui;

i18n!("locales", fallback = "en-US");

fn main() -> iced::Result {
    human_panic::setup_panic!();
    let _guard = log::setup();

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
    Initialized(Result<(Repository, Model), AppError>),
    Ui(ui::Message),
    Mutated(Result<Model, AppError>),
    Deployed,
}

#[derive(Debug, Clone)]
pub struct AppError {
    message: String,
}

#[derive(Debug, Clone)]
enum State {
    Loading,
    Error(AppError),
    Ready {
        repo: Repository,
        model: Model,
        ui: Ui,
    },
}

pub struct App {
    state: State,
    cfg: ConfigStore,
    ui_state: UiStateStore,

    title: String,
}

impl App {
    const TITLE: &str = "Barnacle";

    fn new() -> (Self, Task<Message>) {
        let state = State::Loading;

        let cfg = ConfigStore::new();
        let ui_state = UiStateStore::new();

        (
            Self {
                state,

                cfg,
                ui_state,

                title: Self::TITLE.to_string(),
            },
            Task::perform(
                async {
                    let repo = Repository::new().await;

                    match Model::load(&repo).await {
                        Ok(model) => Ok((repo, model)),
                        Err(error) => {
                            error!(error = %format!("{error:#}"), "failed to initialize app");

                            Err(AppError {
                                message: error.to_string(),
                            })
                        }
                    }
                },
                Message::Initialized,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized(result) => match result {
                Ok((repo, model)) => {
                    let ui = Ui::init(&model, &self.ui_state);

                    self.state = State::Ready { repo, model, ui };

                    Task::none()
                }
                Err(msg) => {
                    self.state = State::Error(msg);

                    Task::none()
                }
            },
            Message::Mutated(result) => match (&mut self.state, result) {
                (State::Ready { model, ui, .. }, Ok(new_model)) => {
                    ui.sync(&new_model);

                    // Reload the model with the fresh data
                    *model = new_model;

                    Task::none()
                }
                (State::Ready { .. }, Err(error)) => {
                    // TODO: Store this in app/ui error state

                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Ui(message) => match &mut self.state {
                State::Ready { repo, ui, .. } => match ui.update(message) {
                    ui::Action::None => Task::none(),
                    ui::Action::Run(task) => task.map(Message::Ui),
                    ui::Action::Mutate(mutation) => {
                        let repo = repo.clone();

                        Task::perform(
                            async move {
                                mutation.run(&repo).await;

                                match Model::load(&repo).await {
                                    Ok(model) => Ok(model),
                                    Err(error) => {
                                        error!("failed to initialize app: {error:#}");

                                        Err(AppError {
                                            message: error.to_string(),
                                        })
                                    }
                                }
                            },
                            Message::Mutated,
                        )
                    }
                    ui::Action::Deploy => {
                        let repo = repo.clone();

                        Task::perform(
                            async move {
                                if let Some(game) = repo.active_game().await.unwrap() {
                                    game.deploy().await;
                                }
                            },
                            |_| Message::Deployed,
                        )
                    }
                },
                _ => Task::none(),
            },
            Message::Deployed => {
                println!("Deployed");

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => text!("LOADING").into(),
            State::Error(_) => panic!("ERROR"),
            State::Ready { model, ui, .. } => ui.view(model).map(Message::Ui),
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.cfg.theme()
    }
}
