use agdb::{CountComparison, DbType, QueryBuilder, QueryId};

use crate::{
    Database, Error, GameCtx, ModCtx, ProfileCtx, Result, UniqueConstraint,
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
    pub async fn insert_profile(
        &mut self,
        new_profile: &Profile,
        game_ctx: GameCtx,
    ) -> Result<ProfileCtx> {
        if self
            .profiles(game_ctx)
            .await?
            .iter()
            .any(|p| p.name() == new_profile.name())
        {
            return Err(Error::UniqueViolation(UniqueConstraint::ProfileName));
        }

        self.0.write().await.transaction_mut(|t| {
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
    pub async fn profiles(&self, game_ctx: GameCtx) -> Result<Vec<Profile>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from(game_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )?
            .try_into()?)
    }

    pub async fn current_profile(&self) -> Result<Profile> {
        let profile: Profile = self
            .0
            .read()
            .await
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

    pub async fn set_current_profile(&mut self, profile_ctx: ProfileCtx) -> Result<()> {
        self.0.write().await.transaction_mut(|t| {
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
    pub async fn insert_mod_entry(
        &mut self,
        mod_ctx: ModCtx,
        profile_ctx: ProfileCtx,
    ) -> Result<()> {
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
            .mod_entries(profile_ctx)
            .await?
            .last()
            .and_then(|e| e.db_id());

        self.0.write().await.transaction_mut(|t| {
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

    pub async fn profile_mods(&self, profile_ctx: ProfileCtx) -> Result<Vec<ProfileMod>> {
        // Traverse the linked-list from the given profile, collecting the ModEntry and Mod nodes.
        let entries = self.mod_entries(profile_ctx).await?;
        let mods: Vec<Mod> = self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Mod>()
                    .search()
                    .from(profile_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .distance(CountComparison::GreaterThan(2))
                    .and()
                    .element::<Mod>()
                    .query(),
            )?
            .try_into()?;

        Ok(entries
            .into_iter()
            .zip(mods)
            .map(|(entry, mod_)| ProfileMod { entry, data: mod_ })
            .collect())
    }

    async fn mod_entries(&self, profile_ctx: ProfileCtx) -> Result<Vec<ModEntry>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<ModEntry>()
                    .search()
                    .from(profile_ctx.id)
                    .where_()
                    .node()
                    .and()
                    .distance(CountComparison::GreaterThan(2))
                    .and()
                    .element::<ModEntry>()
                    .query(),
            )?
            .try_into()?)
    }
}
