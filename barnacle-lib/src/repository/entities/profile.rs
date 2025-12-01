use agdb::{CountComparison, DbId, DbType, QueryBuilder};

use crate::repository::{
    db::DbHandle,
    entities::{Error, Result, get_field, mod_::Mod},
    models::{ModEntryModel, ModModel},
};

/// Represents a profile entity in the Barnacle system.
///
/// Provides methods to inspect and modify this profile's data, including
/// managing mod entries. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Profile {
    id: DbId,
    db: DbHandle,
}

impl Profile {
    pub(crate) fn from_id(id: DbId, db: DbHandle) -> Self {
        Self { id, db }
    }

    pub fn name(&self) -> Result<String> {
        get_field(&self.db, self.id, "name")
    }

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub fn add_mod_entry(&mut self, mod_: Mod) -> Result<()> {
        todo!()
        // let maybe_last_entry_id = self.mod_entries().last().and_then(|e| e.db_id());
        //
        // self.db.write().transaction_mut(|t| -> Result<()> {
        //     let mod_entry = ModEntryModel::default();
        //     let mod_entry_id = t
        //         .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
        //         .elements
        //         .first()
        //         .ok_or(Error::EmptyElements)?
        //         .id;
        //
        //     match maybe_last_entry_id {
        //         Some(last_entry_id) => {
        //             // Connect last entry in list to new entry
        //             t.exec_mut(
        //                 QueryBuilder::insert()
        //                     .edges()
        //                     .from(last_entry_id)
        //                     .to(mod_entry_id)
        //                     .query(),
        //             )?;
        //         }
        //         None => {
        //             // Connect profile node to new entry (first entry in the list)
        //             t.exec_mut(
        //                 QueryBuilder::insert()
        //                     .edges()
        //                     .from(self.id)
        //                     .to(mod_entry_id)
        //                     .query(),
        //             )?;
        //         }
        //     }
        //
        //     // Connect new entry to target mod
        //     t.exec_mut(
        //         QueryBuilder::insert()
        //             .edges()
        //             .from(mod_entry_id)
        //             .to(mod_.id)
        //             .query(),
        //     )?;
        //
        //     Ok(())
        // })?;
        //
        // Ok(())
    }

    pub fn profile_mods(&self) -> Result<Vec<ProfileMod>> {
        todo!()
        // // Traverse the linked-list from the given profile, collecting the ModEntry and Mod nodes.
        // let entries = self.mod_entries();
        // let mods: Vec<ModModel> = self
        //     .db
        //     .read()
        //     .exec(
        //         QueryBuilder::select()
        //             .elements::<ModModel>()
        //             .search()
        //             .from(self.id)
        //             .where_()
        //             .node()
        //             .and()
        //             // Skip the Profile node and the first ModEntry node
        //             .distance(CountComparison::GreaterThan(2))
        //             .query(),
        //     )?
        //     .try_into()
        //     .unwrap();
        //
        // Ok(entries
        //     .into_iter()
        //     .zip(mods)
        //     .map(|(entry, mod_)| ProfileMod::new(entry, mod_))
        //     .collect())
    }

    fn mod_entries(&self) -> Result<Vec<ModEntryModel>> {
        Ok(self
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
                    // Skip the Profile node
                    .distance(CountComparison::GreaterThan(1))
                    .query(),
            )?
            .try_into()
            .unwrap())
    }
}

/// An item in a [`Profile`]'s mod list, containing the [`Mod`] data and its profile-specific configuration.
#[derive(Debug, Clone)]
pub struct ProfileMod {
    // id: Modid,
    entry: ModEntryModel,
    data: ModModel,
}

impl ProfileMod {
    pub fn new(entry: ModEntryModel, data: ModModel) -> Self {
        Self { entry, data }
    }

    pub fn entry(&self) -> &ModEntryModel {
        &self.entry
    }

    pub fn data(&self) -> &ModModel {
        &self.data
    }
}
