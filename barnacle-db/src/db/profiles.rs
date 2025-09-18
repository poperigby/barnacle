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

    pub async fn current_profile(&self) -> Result<Option<ProfileCtx>> {
        let read_guard = self.0.read().await;

        let profile_id = read_guard
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
            .map(|p| p.id);

        let profile_ctx = profile_id.and_then(|id| {
            read_guard
                .exec(
                    QueryBuilder::select()
                        .elements::<Game>()
                        .search()
                        .from("games")
                        .to(id)
                        .where_()
                        .neighbor()
                        .query(),
                )
                .ok()?
                .elements
                .first()
                .map(|g| ProfileCtx { id, game_id: g.id })
        });

        Ok(profile_ctx)
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
                    // Skip the Profile node
                    .distance(CountComparison::GreaterThan(1))
                    .and()
                    .element::<ModEntry>()
                    .query(),
            )?
            .try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeployKind, Game, Mod, Profile};
    use pretty_assertions::assert_eq;
    use tokio::test;

    #[test]
    async fn test_insert_and_list_profiles() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_ctx = db.insert_game(&game).await?;

        let profile = Profile::new("Main");
        let profile_ctx = db.insert_profile(&profile, game_ctx).await?;
        assert_eq!(profile_ctx.game_id, game_ctx.id);

        // Duplicate profile name should fail
        let profile_dup = Profile::new("Main");
        let result = db.insert_profile(&profile_dup, game_ctx).await;
        assert!(matches!(result, Err(Error::UniqueViolation(_))));

        // Listing profiles
        let profiles = db.profiles(game_ctx).await?;
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name(), "Main");

        Ok(())
    }

    #[test]
    async fn test_set_and_get_current_profile() {
        let mut db = Database::new_memory().unwrap();

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_ctx = db.insert_game(&game).await.unwrap();

        let profile = Profile::new("Main");
        let profile_ctx = db.insert_profile(&profile, game_ctx).await.unwrap();

        db.set_current_profile(profile_ctx).await.unwrap();
        let current = db.current_profile().await.unwrap();
        // assert_eq!(current.unwrap().id, profile_ctx.id);
        // assert_eq!(current.unwrap().game_id, game_ctx.id);
    }

    #[test]
    async fn test_insert_single_mod_entry() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_ctx = db.insert_game(&game).await?;
        let profile = Profile::new("Main");
        let profile_ctx = db.insert_profile(&profile, game_ctx).await?;

        let game_mod = Mod::new("Some Mod");
        let mod_ctx = db.insert_mod(&game_mod, game_ctx).await?;
        db.insert_mod_entry(mod_ctx, profile_ctx).await?;

        let profile_mods = db.profile_mods(profile_ctx).await?;
        println!("PROFILE MODS");
        dbg!(&profile_mods);
        assert_eq!(profile_mods.len(), 1);
        assert_eq!(profile_mods[0].data.name(), "Some Mod");

        Ok(())
    }

    #[test]
    async fn test_insert_multiple_mod_entries() -> Result<()> {
        let mut db = Database::new_memory()?;

        let game = Game::new("Skyrim", DeployKind::Gamebryo);
        let game_ctx = db.insert_game(&game).await?;
        let profile = Profile::new("Main");
        let profile_ctx = db.insert_profile(&profile, game_ctx).await?;

        let mod_names = ["ModA", "ModB", "ModC"];
        let mut mod_ctxs = Vec::new();
        for name in &mod_names {
            let game_mod = Mod::new(name);
            mod_ctxs.push(db.insert_mod(&game_mod, game_ctx).await?);
        }

        for mod_ctx in mod_ctxs {
            db.insert_mod_entry(mod_ctx, profile_ctx).await?;
        }

        let profile_mods = db.profile_mods(profile_ctx).await?;
        assert_eq!(profile_mods.len(), mod_names.len());
        for (i, pm) in profile_mods.iter().enumerate() {
            assert_eq!(pm.data.name(), mod_names[i]);
        }

        Ok(())
    }
}
