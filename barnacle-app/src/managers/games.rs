use std::path::Path;

use barnacle_data::v1::games::{DeployType, Game};
use native_db::Database;
use tracing::warn;

pub struct GamesManager<'a> {
    db: &'a Database<'a>,
}

impl<'a> GamesManager<'a> {
    pub fn new(db: &'a Database<'a>) -> Self {
        Self { db }
    }

    pub fn add(&self, name: &str, game_type: DeployType, game_dir: &Path) {
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
