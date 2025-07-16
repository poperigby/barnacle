use barnacle::import;
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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Import { path }) => {
            import::import_mod(path);
        }
        None => {}
    };
}
