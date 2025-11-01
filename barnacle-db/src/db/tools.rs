use agdb::{QueryBuilder, QueryId};

use crate::{GameId, Result, ToolId, db::Database};

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Tool};

impl Database {
    /// Insert a new [`Tool`], linked to the [`Game`] node given by ID
    pub async fn insert_tool(&mut self, new_tool: &Tool, game_id: GameId) -> Result<ToolId> {
        self.0.write().await.transaction_mut(|t| {
            let tool_id = t
                .exec_mut(QueryBuilder::insert().element(&new_tool).query())?
                .elements[0]
                .id;

            // Link Tool to the specified Game node and root "tools" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("tools"), QueryId::from(game_id.0)])
                    .to(tool_id)
                    .query(),
            )?;

            Ok(ToolId(tool_id))
        })
    }

    /// Retrieve [`Tool`]s owned by the [`Game`] given by ID.
    pub async fn tools(&self, game_id: GameId) -> Result<Vec<Tool>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Tool>()
                    .search()
                    .from(game_id.0)
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )?
            .try_into()?)
    }
}
