use std::{fs::create_dir_all, path::PathBuf};

use barnacle::{data_dir, database::Database};
use barnacle_data::v1::{games::Game, mods::Mod, profiles::Profile};
use clap::{Parser, Subcommand};
use native_db::{Builder, Models};
use once_cell::sync::Lazy;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod gui;

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
    // /// Manage profiles
    // Profile {
    //     #[command(subcommand)]
    //     command: Option<ProfileCommands>,
    // },
    // /// Manage mods
    // Mod {
    //     #[command(subcommand)]
    //     command: Option<ModCommands>,
    // },
    Gui,
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
    List,
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

// #[derive(Subcommand)]
// enum ModCommands {
//     /// Add a new mod
//     Add {
//         /// The path to the mod archive to import
//         mod_path: PathBuf,
//         // The game the mod should be imported to
//         game: String,
//         /// Name of the mod to import. This can usually be inferred from the mod archive name.
//         name: Option<String>,
//     },
// }
//
//
//
static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<Game>().unwrap();
    models.define::<Profile>().unwrap();
    models.define::<Mod>().unwrap();
    models
});

fn main() {
    // Setup logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    human_panic::setup_panic!();

    // Make sure data_dir exists
    create_dir_all(data_dir()).unwrap();

    // Setup database
    let db = Database::new(
        Builder::new()
            .create(&MODELS, data_dir().join("state.db"))
            .unwrap(),
    );

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Game {
            command: Some(GameCommands::Add { name, game_dir }),
        }) => {
            // game_manager.add_game(&name, DeployType::Overlay, &game_dir);
            todo!();
        }
        Some(Commands::Game {
            command: Some(GameCommands::List),
        }) => {
            // let r = db.r_transaction().unwrap();
            // let games: Vec<Game> = r
            //     .scan()
            //     .primary()
            //     .unwrap()
            //     .all()
            //     .unwrap()
            //     .collect::<Result<_, _>>()
            //     .unwrap();
            //
            // dbg!(games);
        }
        Some(Commands::Game { command: None }) => {}
        // Some(Commands::Profile {
        //     command: Some(ProfileCommands::Add { name, game }),
        // }) => {
        //     let rw = db.rw_transaction().unwrap();
        //     let game = rw.get().secondary(GameKey, name).unwrap();
        //     game.create_profile(&name);
        // }
        // Some(Commands::Profile { command: None }) => {}
        // Some(Commands::Mod {
        //     command:
        //         Some(ModCommands::Add {
        //             mod_path,
        //             game,
        //             name,
        //         }),
        // }) => {
        //     let game = state.games.iter_mut().find(|g| g.name() == game).unwrap();
        //     game.import_mod(&mod_path, name.as_deref());
        // }
        // Some(Commands::Mod { command: None }) => {}
        Some(Commands::Gui) => {
            gui::run();
        }
        None => {}
    }
}
