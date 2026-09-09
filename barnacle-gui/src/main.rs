//! Entrypoint of the application. This is reponsible for loading the application data and passing
//! it to the main UI that lives in [`Ui`].

use barnacle_lib::Repository;
use fluent_i18n::i18n;
use iced::{Element, Task, Theme, application, widget::text, window::Settings};

use crate::{
    model::Model,
    persistence::{config::ConfigStore, state::UiStateStore},
    ui::Ui,
};

pub mod icons;
pub mod log;
pub mod model;
pub mod persistence;
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
    Initialized(Result<(Repository, Model), String>),
    Ui(ui::Message),
    Mutated(Result<Model, String>),
}

#[derive(Debug, Clone)]
enum State {
    Loading,
    Error(String),
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
                    let model = Model::load(&repo).await.map_err(|e| e.to_string())?;
                    Ok((repo, model))
                },
                Message::Initialized,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized(result) => match result {
                Ok((repo, model)) => {
                    let ui = Ui::init(&self.ui_state);

                    self.state = State::Ready { repo, model, ui };

                    Task::none()
                }
                Err(msg) => {
                    self.state = State::Error(msg);

                    Task::none()
                }
            },
            Message::Mutated(result) => match (&mut self.state, result) {
                (State::Ready { model, .. }, Ok(new_model)) => {
                    // Reload the model with the fresh data
                    *model = new_model;

                    Task::none()
                }
                (State::Ready { .. }, Err(error)) => {
                    // TODO: Store this in app/ui error state
                    eprintln!("{error}");

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
                                Model::load(&repo).await.map_err(|e| e.to_string())
                            },
                            Message::Mutated,
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
