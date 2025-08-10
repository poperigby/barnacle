use std::path::Path;

use native_db::Database;
use tracing::warn;

use crate::domain::v1::games::{DeployType, Game};

pub struct GameRepo<'a> {
    db: Database<'a>,
}

impl<'a> GameRepo<'a> {
    pub fn new(db: Database<'a>) -> Self {
        Self { db }
    }

    pub fn add_game(&self, name: &str, game_type: DeployType, game_dir: &Path) {
        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        let new_game = Game::new(name, game_type, game_dir);

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_game).unwrap();
        rw.commit().unwrap();
    }
}
