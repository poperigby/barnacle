use std::fs::{create_dir_all, remove_dir_all};

use barnacle_db::{
    GameId,
    models::{DeployKind, Game},
};

use crate::{Result, fs::game_dir, repository::Repository};

impl Repository {
    pub async fn add_game(&mut self, name: &str, game_type: DeployKind) -> Result<GameId> {
        let new_game = Game::new(name, game_type);

        create_dir_all(game_dir(&new_game))?;

        Ok(self.db.insert_game(&new_game).await?)
    }

    pub async fn delete_game(&mut self, id: GameId) -> Result<()> {
        let game = self.db.game(id).await?;
        self.db.remove_game(id).await?;

        remove_dir_all(game_dir(&game))?;

        Ok(())
    }

    pub async fn games(&self) -> Result<Vec<Game>> {
        Ok(self.db.games().await?)
    }
}
