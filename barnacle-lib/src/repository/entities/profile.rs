use std::path::PathBuf;

use agdb::{CountComparison, DbId, QueryBuilder};
use heck::ToSnakeCase;

use crate::repository::{
    CoreConfigHandle,
    db::DbHandle,
    entities::{Error, Result, game::Game, get_field, mod_::Mod, mod_entry::ModEntry},
    models::{GameModel, ModEntryModel, ModModel},
};

/// Represents a profile entity in the Barnacle system.
///
/// Provides methods to inspect and modify this profile's data, including
/// managing mod entries. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Profile {
    id: DbId,
    db: DbHandle,
    cfg: CoreConfigHandle,
}

impl Profile {
    pub(crate) fn from_id(id: DbId, db: DbHandle, cfg: CoreConfigHandle) -> Self {
        Self { id, db, cfg }
    }

    // Fields

    pub fn name(&self) -> Result<String> {
        get_field(&self.db, self.id, "name")
    }

    // Utility

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self.parent()?.dir()?.join(self.name()?.to_snake_case()))
    }

    /// Returns the parent [`Game`] of this [`Profile`]
    pub fn parent(&self) -> Result<Game> {
        let parent_game_id = self
            .db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<GameModel>()
                    .search()
                    .from("games")
                    .to(self.id)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .pop()
            .ok_or(Error::EmptyElements)?
            .id;

        Ok(Game::from_id(
            parent_game_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    // Operations

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub fn add_mod_entry(&mut self, mod_: Mod) -> Result<()> {
        let maybe_last_entry_id = self.mod_entries()?.last().map(|e| e.entry_id);

        self.db.write().transaction_mut(|t| -> Result<()> {
            let mod_entry = ModEntryModel::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements
                .first()
                .ok_or(Error::EmptyElements)?
                .id;

            match maybe_last_entry_id {
                Some(last_entry_id) => {
                    // Connect last entry in list to new entry
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(last_entry_id)
                            .to(mod_entry_id)
                            .query(),
                    )?;
                }
                None => {
                    // Connect profile node to new entry (first entry in the list)
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(self.id)
                            .to(mod_entry_id)
                            .query(),
                    )?;
                }
            }

            // Connect new entry to target mod
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(mod_entry_id)
                    .to(mod_.id)
                    .query(),
            )?;

            // TODO: Return ModEntry
            Ok(())
        })?;

        Ok(())
    }

    pub fn mod_entries(&self) -> Result<Vec<ModEntry>> {
        let mod_entry_ids: Vec<DbId> = self
            .db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<ModEntryModel>()
                    .search()
                    .from(self.id)
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )?
            .elements
            .iter()
            .map(|e| e.id)
            .collect();

        let mod_ids: Vec<DbId> = self
            .db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<ModModel>()
                    .search()
                    .from(self.id)
                    .where_()
                    .node()
                    .and()
                    // Skip the Profile node and the first ModEntry node
                    .distance(CountComparison::GreaterThan(2))
                    .query(),
            )?
            .elements
            .iter()
            .map(|e| e.id)
            .collect();

        Ok(mod_entry_ids
            .into_iter()
            .zip(mod_ids)
            .map(|(entry_id, mod_id)| ModEntry::from_id(entry_id, mod_id, self.db.clone()))
            .collect())
    }
}
