use std::fs::{create_dir_all, remove_dir_all};

use barnacle_db::{
    GameId,
    models::{DeployKind, Game},
};

use crate::{Result, repository::Repository};

impl Repository {
    pub async fn add_game(&mut self, name: &str, game_type: DeployKind) -> Result<GameId> {
        let new_game = Game::new(name, game_type);

        create_dir_all(self.cfg().await.core().game_dir(&new_game))?;

        Ok(self.db.write().await.insert_game(&new_game)?)
    }

    pub async fn delete_game(&mut self, id: GameId) -> Result<()> {
        let game = self.db.read().await.game(id)?;
        self.db.write().await.remove_game(id)?;

        remove_dir_all(self.cfg().await.core().game_dir(&game))?;

        Ok(())
    }

    pub async fn games(&self) -> Result<Vec<Game>> {
        Ok(self.db.read().await.games()?)
    }
}
