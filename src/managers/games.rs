use std::{fs::create_dir_all, path::Path};

use native_db::Database;
use tracing::warn;

use crate::{
    data::v1::games::{DeployType, Game},
    data_dir,
};

pub struct GameManager<'a> {
    db: Database<'a>,
}

impl<'a> GameManager<'a> {
    pub fn new(db: Database<'a>) -> Self {
        Self { db }
    }

    pub fn add_game(&self, name: &str, game_type: DeployType, game_dir: &Path) {
        create_dir_all(data_dir().join("profiles").join(name)).unwrap();

        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        let game = Game::new(name, game_type, game_dir);

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(game).unwrap();
        rw.commit().unwrap();
    }
}
