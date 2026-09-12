use std::fmt::Debug;

mod error;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
};
use tracing::info;

pub use error::*;

use crate::repository::{
    Mod, Profile,
    config::Cfg,
    db::{
        Db,
        models::{
            mod_entries::{ActiveModel, COLUMN, Entity, Model},
            mods::{Entity as ModEntity, Model as ModModel},
            profiles::{Entity as ProfileEntity, Model as ProfileModel},
        },
    },
    handles::error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
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

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::ModEntry, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::ModEntry, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

    async fn profile_model(&self) -> Result<ProfileModel, RelatedProfileError> {
        ProfileEntity::find_by_id(
            self.model(self.db.conn())
                .await
                .map_err(RelatedProfileError::LoadEntry)?
                .profile_id,
        )
        .one(self.db.conn())
        .await
        .map_err(RelatedProfileError::LoadProfile)?
        .ok_or(RelatedProfileError::StaleProfile)
    }

    async fn mod_model(&self) -> Result<ModModel, RelatedModError> {
        ModEntity::find_by_id(
            self.model(self.db.conn())
                .await
                .map_err(RelatedModError::LoadEntry)?
                .mod_id,
        )
        .one(self.db.conn())
        .await
        .map_err(RelatedModError::LoadMod)?
        .ok_or(RelatedModError::StaleMod)
    }

    // Fields

    pub async fn name(&self) -> Result<String, NameError> {
        Ok(self.mod_model().await.map_err(NameError)?.name)
    }

    pub async fn enabled(&self) -> Result<bool, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("enabled", source))?
            .enabled)
    }

    pub async fn set_enabled(&self, value: bool) -> Result<(), SetEnabledError> {
        let mut active_model = self
            .active_model(self.db.conn())
            .await
            .map_err(SetEnabledError::Load)?;
        active_model.enabled.set_if_not_equals(value);
        active_model
            .update(self.db.conn())
            .await
            .map_err(SetEnabledError::Update)?;

        Ok(())
    }

    pub async fn notes(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("notes", source))?
            .notes)
    }

    /// Returns the parent [`Profile`] of this [`ModEntry`]
    pub async fn parent(&self) -> Result<Profile, ParentError> {
        Ok(Profile::from_id(
            self.model(self.db.conn())
                .await
                .map_err(ParentError::Load)?
                .profile_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    pub(crate) async fn add(
        db: &Db,
        cfg: &Cfg,
        profile: &Profile,
        mod_: Mod,
    ) -> Result<Self, AddError> {
        let next_priority = Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .order_by_desc(COLUMN.priority)
            .one(db.conn())
            .await
            .map_err(AddError::NextPriority)?
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
            .map_err(|source| {
                if is_unique_violation(&source) {
                    AddError::Duplicate
                } else {
                    AddError::Insert(source)
                }
            })?
            .last_insert_id;
        let mod_entry = ModEntry::from_id(id, db.clone(), cfg.clone());

        info!(
            "Added mod entry for mod '{}' to profile '{}'",
            mod_.name().await.map_err(AddError::ModName)?,
            profile.name().await.map_err(AddError::ProfileName)?
        );

        Ok(mod_entry)
    }

    /// Remove the given [`ModEntry`] from the list
    pub async fn remove(self) -> Result<(), RemoveError> {
        let mod_name = self.mod_model().await.map_err(RemoveError::ModName)?.name;
        let profile_name = self
            .profile_model()
            .await
            .map_err(RemoveError::ProfileName)?
            .name;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        info!(
            "Removed mod entry for mod '{}' from profile '{}'",
            mod_name, profile_name
        );

        Ok(())
    }

    pub(crate) async fn list(
        db: &Db,
        cfg: &Cfg,
        profile: &Profile,
    ) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .all(db.conn())
            .await
            .map_err(ListError)?
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
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

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
