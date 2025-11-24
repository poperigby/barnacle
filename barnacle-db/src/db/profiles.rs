use agdb::{CountComparison, DbType, QueryBuilder, QueryId};

use crate::{
    Database, Error, GameId, ModId, ProfileId, Result, UniqueConstraint,
    models::{ModEntry, Profile},
};

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Mod};

impl Database {
    /// Insert a new [`Profile`], linked to the [`Game`] node given by ID. The [`Profile`] name must be unique.
    pub fn insert_profile(&mut self, new_profile: &Profile, game_id: GameId) -> Result<ProfileId> {
        if self
            .profiles(game_id)?
            .iter()
            .any(|p| p.name() == new_profile.name())
        {
            return Err(Error::UniqueViolation(UniqueConstraint::ProfileName));
        }

        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(new_profile).query())?
                .elements
                .first()
                .ok_or(Error::EmptyInsertResult)?
                .id;

            // Link Profile to the specified Game node and root "profiles" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(game_id.0)])
                    .to(profile_id)
                    .query(),
            )?;

            Ok(ProfileId(profile_id))
        })
    }

    /// Retrieve [`Profile`]s owned by the [`Game`] given by ID.
    pub fn profiles(&self, game_id: GameId) -> Result<Vec<Profile>> {
        Ok(self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
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

    pub fn current_profile(&self) -> Result<Option<ProfileId>> {
        let profile_id = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from("current_profile")
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .first()
            .map(|p| ProfileId(p.id));

        Ok(profile_id)
    }

    pub fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // Delete existing current_profile, if it exists
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .from("current_profile")
                    .where_()
                    .edge()
                    .query(),
            )?;
            // Insert a new edge from current_profile to new profile_id
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from("current_profile")
                    .to(profile_id.0)
                    .query(),
            )?;

            Ok(())
        })
    }

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub fn insert_mod_entry(&mut self, mod_id: ModId, profile_id: ProfileId) -> Result<()> {
        /*
        ModEntry nodes are stored in a linked list like data structure:
                                   Game1
                                     │
                          ┌──────────┼──────────┐
                          │          │          │
                          ↓          ↓          ↓
                       Mod1        Mod2        Mod3
                          ↑          ↑          ↑
                          │          │          │
        Profile1 ──→ ModEntry1 ──→ ModEntry2 ──→ ModEntry3
        */

        let maybe_last_entry_id = self.mod_entries(profile_id)?.last().and_then(|e| e.db_id());

        self.0.transaction_mut(|t| {
            // Insert new ModEntry
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements
                .first()
                .ok_or(Error::EmptyInsertResult)?
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
                            .from(profile_id.0)
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
                    .to(mod_id.0)
                    .query(),
            )?;

            Ok(())
        })
    }

    pub fn profile_mods(&self, profile_id: ProfileId) -> Result<Vec<ProfileMod>> {
        // Traverse the linked-list from the given profile, collecting the ModEntry and Mod nodes.
        let entries = self.mod_entries(profile_id)?;
        let mods: Vec<Mod> = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Mod>()
                    .search()
                    .from(profile_id.0)
                    .where_()
                    .node()
                    .and()
                    // Skip the Profile node and the first ModEntry node
                    .distance(CountComparison::GreaterThan(2))
                    .and()
                    .element::<Mod>()
                    .query(),
            )?
            .try_into()?;

        Ok(entries
            .into_iter()
            .zip(mods)
            .map(|(entry, mod_)| ProfileMod::new(entry, mod_))
            .collect())
    }

    fn mod_entries(&self, profile_id: ProfileId) -> Result<Vec<ModEntry>> {
        Ok(self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<ModEntry>()
                    .search()
                    .from(profile_id.0)
                    .where_()
                    .node()
                    .and()
                    // Skip the Profile node
                    .distance(CountComparison::GreaterThan(1))
                    .and()
                    .element::<ModEntry>()
                    .query(),
            )?
            .try_into()?)
    }
}

/// An item in a [`Profile`]'s mod list, containing the [`Mod`] data and its profile-specific configuration.
#[derive(Debug, Clone)]
pub struct ProfileMod {
    // id: Modid,
    entry: ModEntry,
    data: Mod,
}

impl ProfileMod {
    pub fn new(entry: ModEntry, data: Mod) -> Self {
        Self { entry, data }
    }

    pub fn entry(&self) -> &ModEntry {
        &self.entry
    }

    pub fn data(&self) -> &Mod {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::{DeployKind, Game, Mod, Profile};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_insert_and_list_profiles() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_id = db.insert_game(&game)?;

        let profile = Profile::new("Main");
        let profile_id = db.insert_profile(&profile, game_id)?;

        // Duplicate profile name should fail
        let profile_dup = Profile::new("Main");
        let result = db.insert_profile(&profile_dup, game_id);
        assert!(matches!(result, Err(Error::UniqueViolation(_))));

        // Listing profiles
        let profiles = db.profiles(game_id)?;
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles.first().unwrap().name(), "Main");

        Ok(())
    }

    #[test]
    fn test_set_and_get_current_profile() {
        let mut db = Database::new_memory().unwrap();

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_id = db.insert_game(&game).unwrap();

        let profile = Profile::new("Main");
        let profile_id = db.insert_profile(&profile, game_id).unwrap();

        db.set_current_profile(profile_id).unwrap();
        let current = db.current_profile().unwrap();
        // assert_eq!(current.unwrap().id, profile_id.id);
        // assert_eq!(current.unwrap().game_id, game_id.id);
    }

    #[test]
    fn test_insert_single_mod_entry() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_id = db.insert_game(&game)?;
        let profile = Profile::new("Main");
        let profile_id = db.insert_profile(&profile, game_id)?;

        let game_mod = Mod::new("Some Mod");
        let mod_id = db.insert_mod(&game_mod, game_id)?;
        db.insert_mod_entry(mod_id, profile_id)?;

        let profile_mods = db.profile_mods(profile_id)?;
        assert_eq!(profile_mods.len(), 1);
        assert_eq!(profile_mods.first().unwrap().data().name(), "Some Mod");

        Ok(())
    }

    #[test]
    fn test_insert_multiple_mod_entries() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_id = db.insert_game(&game)?;
        let profile = Profile::new("Main");
        let profile_id = db.insert_profile(&profile, game_id)?;

        let mod_names = ["ModA", "ModB", "ModC"];
        let mut mod_ids = Vec::new();
        for name in &mod_names {
            let game_mod = Mod::new(name);
            mod_ids.push(db.insert_mod(&game_mod, game_id)?);
        }

        for mod_id in mod_ids {
            db.insert_mod_entry(mod_id, profile_id)?;
        }

        let profile_mods = db.profile_mods(profile_id)?;
        assert_eq!(profile_mods.len(), mod_names.len());
        for (i, pm) in profile_mods.iter().enumerate() {
            assert_eq!(&pm.data().name(), mod_names.get(i).unwrap());
        }

        Ok(())
    }
}
