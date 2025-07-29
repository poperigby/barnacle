use barnacle::{
    games::{Game, GameType},
    overlay::Overlay,
    state_file::State,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
    /// Manage mods
    Mod {
        #[command(subcommand)]
        command: Option<ModCommands>,
    },
}

#[derive(Subcommand)]
enum GameCommands {
    /// Add a new game
    Add {
        /// Name of the game to add
        name: String,
        /// Path to the game directory
        game_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Add a new profile
    Add {
        /// Name of the profile to add
        name: String,
        // The game the profile should be added to
        game: String,
    },
}

#[derive(Subcommand)]
enum ModCommands {
    /// Add a new mod
    Add {
        /// The path to the mod archive to import
        mod_path: PathBuf,
        // The game the mod should be imported to
        game: String,
        /// Name of the mod to import. This can usually be inferred from the mod archive name.
        name: Option<String>,
    },
}

fn main() {
    // Setup logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Load configuration file
    let mut state = State::load().unwrap();

    // Load overlay
    let game = state.games.first().unwrap();
    let profile = game.profiles().first().unwrap();
    let mut overlay = Overlay::new(game, profile);
    overlay.mount();

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Game {
            command: Some(GameCommands::Add { name, game_dir }),
        }) => {
            let game = Game::new(&name, GameType::Generic, &game_dir);
            state.games.push(game);
        }
        Some(Commands::Game { command: None }) => {}
        Some(Commands::Profile {
            command: Some(ProfileCommands::Add { name, game }),
        }) => {
            let game = state.games.iter_mut().find(|g| g.name() == game).unwrap();
            game.create_profile(&name);
        }
        Some(Commands::Profile { command: None }) => {}
        Some(Commands::Mod {
            command:
                Some(ModCommands::Add {
                    mod_path,
                    game,
                    name,
                }),
        }) => {
            let game = state.games.iter_mut().find(|g| g.name() == game).unwrap();
            game.import_mod(&mod_path, name.as_deref());
        }
        Some(Commands::Mod { command: None }) => {}
        None => {}
    }

    state.save().unwrap();
}
