use agdb::QueryBuilder;

use crate::{
    db::{Database, GameId, ModId, Result},
    schema::v1::mods::Mod,
};

impl Database {
    /// Insert a new Mod, linked to the given Game node
    pub fn insert_mod(&mut self, new_mod: &Mod, game_id: GameId) -> Result<ModId> {
        self.0.transaction_mut(|t| {
            let mod_id = t
                .exec_mut(QueryBuilder::insert().element(new_mod).query())?
                .elements[0]
                .id;

            // Link Mod to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id.0)
                    .to(mod_id)
                    .query(),
            )?;
            Ok(ModId(mod_id))
        })
    }
}
