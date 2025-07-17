use barnacle::{config::Config, games::Game, import};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Import a mod archive or directory
    Import {
        /// Path to the mod to import
        path: PathBuf,
    },
    /// Manage games
    Game {
        #[command(subcommand)]
        command: Option<GameCommands>,
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

// #[derive(Subcommand)]
// enum ProfileCommands {
//     Add {
//         /// Name of the profile to add
//         name: String,
//         game:
//     },
// }

fn main() {
    let cli = Cli::parse();
    let mut config = Config::load().unwrap();

    match &cli.command {
        Some(Commands::Import { path }) => import::import_mod(path),
        Some(Commands::Game {
            command: Some(GameCommands::Add { name, game_dir }),
        }) => {
            let game = Game::new(name.into(), game_dir.into());
            config.games.push(game);
        }
        Some(Commands::Game { command: None }) => {}
        None => {}
    }

    config.save().unwrap();
}
