use barnacle_lib::{Repository, repository::DeployKind};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// List games
    List,
    /// Add a new game
    Add { name: String },
    /// Remove the given game
    Remove { name: String },
    /// Activate the given game
    Activate { name: String },
}

pub async fn handle(repo: &Repository, cmd: &Command) {
    match cmd {
        Command::List => {
            let games = repo.games().await.unwrap();
            for game in games {
                println!("{}", game.name().await.unwrap());
            }
        }
        Command::Add { name } => {
            repo.add_game(name, DeployKind::Overlay).await.unwrap();
        }
        Command::Remove { name } => {
            let game = repo
                .search_game(name)
                .await
                .unwrap()
                .expect("game not found");

            game.remove().await.unwrap();
        }
        Command::Activate { name } => {
            let game = repo
                .search_game(name)
                .await
                .unwrap()
                .expect("game not found");

            game.activate().await.unwrap();
        }
    }
}
