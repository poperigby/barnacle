use agdb::{CountComparison, DbType, QueryBuilder, QueryId};

use crate::{
    Database, DatabaseError, GameCtx, ModCtx, ProfileCtx, Result, UniqueConstraint,
    models::{ModEntry, Profile},
};

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Mod};

/// An item in a [`Profile`]'s mod list, containing the [`Mod`] data and its profile-specific configuration.
#[derive(Debug, Clone)]
pub struct ProfileMod {
    // ctx: ModCtx,
    entry: ModEntry,
    data: Mod,
}

impl Database {
    /// Insert a new [`Profile`], linked to the [`Game`] node given by ID. The [`Profile`] name must be unique.
    pub fn insert_profile(
        &mut self,
        new_profile: &Profile,
        game_ctx: GameCtx,
    ) -> Result<ProfileCtx> {
        if self
            .profiles(game_ctx)?
            .iter()
            .any(|p| p.name() == new_profile.name())
        {
            return Err(DatabaseError::UniqueViolation(
                UniqueConstraint::ProfileName,
            ));
        }

        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(new_profile).query())?
                .elements[0]
                .id;

            // Link Profile to the specified Game node and root "profiles" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(game_ctx.id)])
                    .to(profile_id)
                    .query(),
            )?;

            Ok(ProfileCtx {
                id: profile_id,
                game_id: game_ctx.id,
            })
        })
    }

    /// Retrieve [`Profile`]s owned by the [`Game`] given by ID.
    pub fn profiles(&self, game_ctx: GameCtx) -> Result<Vec<Profile>> {
        Ok(self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from(game_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .distance(CountComparison::GreaterThan(1))
                    .query(),
            )?
            .try_into()?)
    }

    pub fn current_profile(&self) -> Result<Profile> {
        let profile: Profile = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from("current_profile")
                    .query(),
            )?
            .try_into()?;

        Ok(profile)
    }

    pub fn set_current_profile(&mut self, profile_ctx: ProfileCtx) -> Result<()> {
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
                    .to(profile_ctx.id)
                    .query(),
            )?;

            Ok(())
        })
    }

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub fn insert_mod_entry(&mut self, mod_ctx: ModCtx, profile_ctx: ProfileCtx) -> Result<()> {
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

        // TODO: Replace with real error
        assert_eq!(mod_ctx.game_id, profile_ctx.game_id);

        let maybe_last_entry_id = self
            .mod_entries(profile_ctx)?
            .last()
            .and_then(|e| e.db_id());

        self.0.transaction_mut(|t| {
            // Insert new ModEntry
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements[0]
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
                            .from(profile_ctx.id)
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
                    .to(mod_ctx.id)
                    .query(),
            )?;

            Ok(())
        })
    }

    pub fn profile_mods(&self, profile_ctx: ProfileCtx) -> Result<Vec<ProfileMod>> {
        // Traverse the linked-list from the given profile, collecting the ModEntry and Mod nodes.
        let entries = self.mod_entries(profile_ctx)?;
        let mods: Vec<Mod> = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Mod>()
                    .search()
                    .from(profile_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .distance(CountComparison::GreaterThan(1))
                    .and()
                    .keys(Mod::db_keys())
                    .query(),
            )?
            .try_into()?;

        Ok(entries
            .into_iter()
            .zip(mods)
            .map(|(entry, mod_)| ProfileMod { entry, data: mod_ })
            .collect())
    }

    fn mod_entries(&self, profile_ctx: ProfileCtx) -> Result<Vec<ModEntry>> {
        Ok(self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<ModEntry>()
                    .search()
                    .from(profile_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .distance(CountComparison::GreaterThan(1))
                    .and()
                    .keys(ModEntry::db_keys())
                    .query(),
            )?
            .try_into()?)
    }
}
