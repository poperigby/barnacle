//! Entrypoint of the application. This is reponsible for loading the application data and passing
//! it to the main UI that lives in [`Ui`].

use barnacle_gui::{
    log,
    model::Model,
    persistence::{config::ConfigStore, state::UiStateStore},
};
use barnacle_l10n::Translations;
use barnacle_lib::Repository;
use iced::{Element, Task, Theme, application, widget::text, window::Settings};
use tracing::error;

use crate::ui::Ui;

pub mod ui;

fn main() -> iced::Result {
    human_panic::setup_panic!();
    let _log_guard = log::setup();

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
    Initialized(Result<(Repository, Model, Translations), AppError>),
    Reloaded(Result<Model, AppError>),
    Mutated(Result<(), String>),
    Launched,
    Ui(ui::Message),
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
        translations: Translations,
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
                    let translations = repo.translations().await.unwrap();

                    match Model::load(&repo).await {
                        Ok(model) => Ok((repo, model, translations)),
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
                Ok((repo, model, translations)) => {
                    let ui = Ui::init(&model, &self.ui_state);

                    self.state = State::Ready {
                        repo,
                        model,
                        translations,
                        ui,
                    };

                    Task::none()
                }
                Err(msg) => {
                    self.state = State::Error(msg);

                    Task::none()
                }
            },
            Message::Reloaded(result) => match (&mut self.state, result) {
                (State::Ready { model, ui, .. }, Ok(new_model)) => {
                    ui.sync(&new_model);

                    *model = new_model;

                    Task::none()
                }
                (State::Loading, Ok(_)) => todo!(),
                (State::Loading, Err(_)) => todo!(),
                (State::Error(app_error), Ok(_)) => todo!(),
                (State::Error(app_error), Err(_)) => todo!(),
                (
                    State::Ready {
                        repo,
                        model,
                        translations,
                        ui,
                    },
                    Err(_),
                ) => todo!(),
            },
            Message::Mutated(result) => match result {
                Ok(()) => {
                    let State::Ready { repo, .. } = &self.state else {
                        return Task::none();
                    };

                    let repo = repo.clone();
                    Task::perform(
                        async move {
                            Model::load(&repo).await.map_err(|e| AppError {
                                message: e.to_string(),
                            })
                        },
                        Message::Reloaded,
                    )
                }
                Err(e) => todo!("Store somewhere"),
            },
            Message::Launched => {
                println!("Launched!");

                Task::none()
            }
            Message::Ui(message) => match &mut self.state {
                State::Ready { repo, ui, .. } => match ui.update(message) {
                    ui::Action::None => Task::none(),
                    ui::Action::Run(task) => task.map(Message::Ui),
                    ui::Action::Mutate(mutation) => mutation.run(repo, |result| {
                        Message::Mutated(result.map_err(|e| e.to_string()))
                    }),
                    ui::Action::Launch => {
                        let repo = repo.clone();

                        Task::perform(
                            async move {
                                if let Some(game) = repo.active_game().await.unwrap() {
                                    game.launch().await;
                                }
                            },
                            |_| Message::Launched,
                        )
                    }
                },
                _ => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => text!("LOADING").into(),
            State::Error(_) => panic!("ERROR"),
            State::Ready {
                model,
                translations,
                ui,
                ..
            } => ui.view(model, translations).map(Message::Ui),
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.cfg.theme()
    }
}
