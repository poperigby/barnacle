use agdb::{CountComparison, QueryBuilder, QueryId};

use crate::{
    Database, DatabaseError, GameCtx, ModCtx, ProfileCtx, Result, UniqueConstraint,
    models::{ModEntry, Profile},
};

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Mod};

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
        // TODO: Replace with real error
        assert_eq!(mod_ctx.game_id, profile_ctx.game_id);

        self.0.transaction_mut(|t| {
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements[0]
                .id;

            // ModEntry nodes are stored in a linked list like data structure:
            // Profile -> ModEntry1 -> ModEntry2 -> ModEntry3
            // Each ModEntry points to a Mod under a Game

            // TODO:
            // Find last ModEntry in the list
            // Append new ModEntry to that node

            // Insert ModEntry in between Profile and Mod
            // Profile -> ModEntry -> Mod
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(profile_ctx.id)
                    .to(mod_entry_id)
                    .query(),
            )?;
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

    pub fn mod_entries(&self, profile_ctx: ProfileCtx) -> Result<Vec<ModEntry>> {
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
                    .query(),
            )?
            .try_into()?)
    }
}
