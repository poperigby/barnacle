use std::fmt::Debug;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryFilter, QueryOrder};
use tracing::info;

use crate::repository::{
    Mod, Profile,
    config::Cfg,
    db::{
        Db,
        models::{
            mod_entries::{ActiveModel, COLUMN, Entity, Model},
            mods::{Entity as ModEntity, Model as ModModel},
        },
    },
    handles::{Error, Result, map_insert_error},
};

/// Represents a mod entry in the Barnacle system.
///
/// Provides methods to inspect and modify this mod entry's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct ModEntry {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl ModEntry {
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
    }

    async fn model(&self) -> Result<Model> {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    async fn active_model(&self) -> Result<ActiveModel> {
        Ok(self.model().await?.into())
    }

    async fn profile_model(&self) -> Result<ModModel> {
        ModEntity::find_by_id(self.model().await?.profile_id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    async fn mod_model(&self) -> Result<ModModel> {
        ModEntity::find_by_id(self.model().await?.mod_id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    // Fields

    pub async fn name(&self) -> Result<String> {
        Ok(self.mod_model().await?.name)
    }

    pub async fn enabled(&self) -> Result<bool> {
        Ok(self.model().await?.enabled)
    }

    pub async fn set_enabled(&self, value: bool) -> Result<()> {
        let mut active_model = self.active_model().await?;
        active_model.enabled.set_if_not_equals(value);
        active_model.update(self.db.conn()).await?;

        Ok(())
    }

    pub async fn notes(&self) -> Result<String> {
        Ok(self.model().await?.notes)
    }

    /// Returns the parent [`Profile`] of this [`ModEntry`]
    pub async fn parent(&self) -> Result<Profile> {
        Ok(Profile::from_id(
            self.model().await?.profile_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, profile: &Profile, mod_: Mod) -> Result<Self> {
        let next_priority = Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .order_by_desc(COLUMN.priority)
            .one(db.conn())
            .await?
            .map_or(0, |e| e.priority + 1);

        let model = ActiveModel {
            profile_id: Set(profile.id),
            mod_id: Set(mod_.id),
            priority: Set(next_priority),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|e| map_insert_error(e, Error::DuplicateModEntry))?
            .last_insert_id;
        let mod_entry = ModEntry::from_id(id, db.clone(), cfg.clone());

        info!(
            "Added mod entry for mod '{}' to profile '{}'",
            mod_.name().await?,
            profile.name().await?
        );

        Ok(mod_entry)
    }

    /// Remove the given [`ModEntry`] from the list
    pub async fn remove(self) -> Result<()> {
        let mod_name = self.mod_model().await?.name;
        let profile_name = self.profile_model().await?.name;

        Entity::delete_by_id(self.id).exec(self.db.conn()).await?;

        info!(
            "Removed mod entry for mod '{}' from profile '{}'",
            mod_name, profile_name
        );

        Ok(())
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, profile: &Profile) -> Result<Vec<Self>> {
        Ok(Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .all(db.conn())
            .await?
            .iter()
            .map(|model| ModEntry::from_id(model.id, db.clone(), cfg.clone()))
            .collect())
    }
}

impl PartialEq for ModEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Repository, repository::DeployKind};

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        let mod1 = game.add_mod("Super Duper Mod", None).await.unwrap();
        let mod2 = game.add_mod("Super Duper Mod: 2", None).await.unwrap();

        profile.add_mod_entry(mod1).await.unwrap();
        profile.add_mod_entry(mod2).await.unwrap();

        assert_eq!(profile.mod_entries().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        let mut mod_entries = Vec::new();
        for i in 1..=6 {
            let m = game.add_mod(&format!("Mod{i}"), None).await.unwrap();
            mod_entries.push(profile.add_mod_entry(m).await.unwrap());
        }

        assert_eq!(profile.mod_entries().await.unwrap().len(), 6);

        async fn remove_and_check(entry: &ModEntry, profile: &Profile) {
            entry.clone().remove().await.unwrap();
            let entries = profile.mod_entries().await.unwrap();
            assert!(!entries.contains(entry));
        }

        remove_and_check(mod_entries.first().unwrap(), &profile).await; // first
        remove_and_check(mod_entries.get(3).unwrap(), &profile).await; // middle
        remove_and_check(mod_entries.get(5).unwrap(), &profile).await; // last

        // Check remaining entries are exactly the ones we expect
        let remaining: Vec<&ModEntry> = mod_entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match i {
                // Filter out the entries we removed
                0 | 3 | 5 => None,
                // These are the ones we expect to be here
                _ => Some(e),
            })
            .collect();
        assert_eq!(
            profile
                .mod_entries()
                .await
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            remaining
        );
    }

    #[tokio::test]
    async fn test_parent() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("The Best Profile").await.unwrap();
        let mod_ = game
            .add_mod(
                "Better Khajiit Balls 16K - Remastered - 2025 Edition - REAL",
                None,
            )
            .await
            .unwrap();
        let entry = profile.add_mod_entry(mod_).await.unwrap();

        assert_eq!(entry.parent().await.unwrap(), profile);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();
        let mod_ = game.add_mod("Super Duper Mod", None).await.unwrap();

        profile
            .add_mod_entry(mod_)
            .await
            .unwrap()
            .name()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_enabled() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();
        let mod_ = game.add_mod("Super Duper Mod", None).await.unwrap();

        let entry = profile.add_mod_entry(mod_).await.unwrap();

        assert!(entry.enabled().await.unwrap());

        entry.set_enabled(false).await.unwrap();

        assert!(!entry.enabled().await.unwrap());
    }
}
