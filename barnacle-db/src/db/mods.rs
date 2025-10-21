use agdb::{QueryBuilder, QueryId};

use crate::{GameId, ModId, Result, db::Database, models::Mod};

// Documentation imports
#[allow(unused_imports)]
use crate::models::Game;

impl Database {
    /// Insert a new [`Mod`], linked to the [`Game`] node given by ID
    pub async fn insert_mod(&mut self, new_mod: &Mod, game_id: GameId) -> Result<ModId> {
        self.0.write().await.transaction_mut(|t| {
            let mod_id = t
                .exec_mut(QueryBuilder::insert().element(&new_mod).query())?
                .elements[0]
                .id;

            // Link Mod to the specified Game node and root "mods" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("mods"), QueryId::from(game_id.0)])
                    .to(mod_id)
                    .query(),
            )?;

            Ok(ModId(mod_id))
        })
    }
}
