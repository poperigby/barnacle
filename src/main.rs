use barnacle::{config::Config, games::Game};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage games
    Game {
        #[command(subcommand)]
        command: Option<GameCommands>,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        command: Option<ProfileCommands>,
    },
}

#[derive(Subcommand)]
enum GameCommands {
    Add {
        /// Name of the game to add
        name: String,
        /// Path to the game directory
        game_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    Add {
        /// Name of the profile to add
        name: String,
        // Name of the game the profile should be added to
        game: String,
    },
}

fn main() {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Load configuration file
    let mut config = Config::load().unwrap();

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Game {
            command: Some(GameCommands::Add { name, game_dir }),
        }) => {
            let game = Game::new(name.into(), game_dir.into());
            config.games.push(game);
        }
        Some(Commands::Game { command: None }) => {}
        Some(Commands::Profile {
            command: Some(ProfileCommands::Add { name, game }),
        }) => {
            let game = config.games.iter_mut().find(|g| g.name() == game).unwrap();
            game.create_profile(name.into());
        }
        Some(Commands::Profile { command: None }) => {}
        None => {}
    }

    config.save().unwrap();
}
