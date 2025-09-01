use agdb::{CountComparison, QueryBuilder, QueryId};

use crate::{
    Database, DatabaseError, GameId, ModId, ProfileId, Result, UniqueConstraint,
    models::{ModEntry, Profile},
};

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Mod};

impl Database {
    /// Insert a new [`Profile`], linked to the [`Game`] node given by ID. The [`Profile`] name must be unique.
    pub fn insert_profile(&mut self, profile: &Profile, game_id: &GameId) -> Result<ProfileId> {
        if self
            .profiles(game_id)?
            .iter()
            .any(|p| p.name() == profile.name())
        {
            return Err(DatabaseError::UniqueViolation(
                UniqueConstraint::ProfileName,
            ));
        }

        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(profile).query())?
                .elements[0]
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
    pub fn profiles(&self, game_id: &GameId) -> Result<Vec<Profile>> {
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

    pub fn set_current_profile(&mut self, profile_id: &ProfileId) -> Result<()> {
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
    pub fn link_mod_to_profile(&mut self, mod_id: &ModId, profile_id: &ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements[0]
                .id;

            // Insert ModEntry in between Profile and Mod
            // Profile -> ModEntry -> Mod
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(profile_id.0)
                    .to(mod_entry_id)
                    .query(),
            )?;
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

    pub fn mod_entries(&self, profile_id: &ProfileId) -> Result<Vec<ModEntry>> {
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
                    .distance(CountComparison::GreaterThan(1))
                    .query(),
            )?
            .try_into()?)
    }
}
