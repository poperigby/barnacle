use std::path::Path;

use barnacle_lib::Repository;
use clap::Subcommand;
use sysexits::ExitCode;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// List profiles
    List,
    /// Add a new profile
    Add { name: String, path: Option<String> },
}

pub async fn handle(repo: &Repository, cmd: &Command) {
    if let Some(active_game) = repo.active_game().await.unwrap() {
        if let Some(active_profile) = active_game.active_profile().await.unwrap() {
            match cmd {
                Command::List => {
                    let mods = active_profile.mod_entries().await.unwrap();
                    for mod_ in mods {
                        println!("* {}", mod_.name().await.unwrap());
                    }
                }
                Command::Add { name, path } => {
                    let mod_ = active_game
                        .add_mod(name, path.as_deref().map(Path::new))
                        .await
                        .unwrap();
                    active_profile.add_mod_entry(mod_).await.unwrap();
                }
            }
        } else {
            eprintln!("No active profile");
            ExitCode::Usage.exit()
        }
    } else {
        eprintln!("No active game");
        ExitCode::Usage.exit()
    }
}
