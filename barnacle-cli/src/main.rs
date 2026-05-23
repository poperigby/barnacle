use barnacle_lib::Repository;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod game;
mod mod_;
mod profile;

#[derive(Parser, Debug)]
#[command(name = "barnacle")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Override the active game
    #[arg(short, long, global = true)]
    game: Option<String>,

    /// Override the active profile
    #[arg(short, long, global = true)]
    profile: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Operate on games
    #[command(subcommand)]
    Game(game::Command),
    /// Operate on profiles
    #[command(subcommand)]
    Profile(profile::Command),
    /// Operate on mods
    #[command(subcommand)]
    Mod(mod_::Command),
}

#[tokio::main]
async fn main() {
    human_panic::setup_panic!();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let repo = Repository::new().await;
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => match cmd {
            Command::Game(cmd) => game::handle(&repo, cmd).await,
            Command::Profile(cmd) => profile::handle(&repo, cmd).await,
            Command::Mod(cmd) => mod_::handle(&repo, cmd).await,
        },
        None => status(&repo).await,
    }
}

async fn status(repo: &Repository) {
    let active_game = match repo.active_game().await.unwrap() {
        Some(game) => game.name().await.unwrap().green(),
        None => "None".red(),
    };

    println!(
        r#"
Active game: {}
        "#,
        active_game
    )
}
