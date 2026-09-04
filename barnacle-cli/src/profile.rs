use barnacle_lib::Repository;
use clap::Subcommand;
use sysexits::ExitCode;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// List profiles
    List,
    /// Add a new profile
    Add { name: String },
    /// Activate the given profile
    Activate { name: String },
}

pub async fn handle(repo: &Repository, cmd: &Command) {
    if let Some(active_game) = repo.active_game().await.unwrap() {
        match cmd {
            Command::List => {
                let profiles = active_game.profiles().await.unwrap();
                for profile in profiles {
                    println!("* {}", profile.name().await.unwrap())
                }
            }
            Command::Add { name } => {
                active_game.add_profile(name).await.unwrap();
            }
            Command::Activate { name } => {
                let profile = active_game
                    .search_profile(name)
                    .await
                    .unwrap()
                    .expect("profile not found");
                profile.activate().await.unwrap();
            }
        }
    } else {
        println!("No active game");
        ExitCode::Usage.exit()
    }
}
