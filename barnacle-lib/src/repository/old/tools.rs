use std::path::Path;

use barnacle_db::{GameId, ToolId, models::Tool};

use crate::{Result, repository::Repository};

impl Repository {
    pub async fn add_tool(
        &mut self,
        game_id: GameId,
        name: &str,
        path: &Path,
        args: Option<&str>,
    ) -> Result<ToolId> {
        let new_tool = Tool::new(name, path.to_path_buf(), args);

        Ok(self.db.write().await.insert_tool(&new_tool, game_id)?)
    }

    pub async fn tools(&self, game_id: GameId) -> Result<Vec<Tool>> {
        Ok(self.db.read().await.tools(game_id)?)
    }
}
