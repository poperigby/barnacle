pub(crate) mod schema;

pub(crate) use schema::*;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    QueryOrder,
};

use crate::repository::{
    Cfg, Mod, Profile,
    db::Db,
    entities::{Error, Result, mods},
};

#[derive(Debug, Clone)]
pub struct ModEntry {
    pub(crate) entry_id: i64,
    pub(crate) mod_id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl ModEntry {
    pub(crate) async fn load(entry_row_id: i64, mod_row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let entry_model = Entity::find_by_id(entry_row_id).one(db.conn()).await?;
        let mod_model = mods::schema::Entity::find_by_id(mod_row_id)
            .one(db.conn())
            .await?;

        let Some(entry_model) = entry_model else {
            return Err(Error::RemovedEntity);
        };
        let Some(mod_model) = mod_model else {
            return Err(Error::RemovedEntity);
        };

        Ok(Self {
            entry_id: entry_model.id,
            mod_id: mod_model.id,
            db,
            cfg,
        })
    }

    async fn entry_model(&self) -> Result<Model> {
        let model = Entity::find_by_id(self.entry_id)
            .one(self.db.conn())
            .await?;
        model.ok_or(Error::RemovedEntity)
    }

    async fn mod_model(&self) -> Result<mods::schema::Model> {
        let model = mods::schema::Entity::find_by_id(self.mod_id)
            .one(self.db.conn())
            .await?;
        model.ok_or(Error::RemovedEntity)
    }

    pub async fn name(&self) -> Result<String> {
        Ok(self.mod_model().await?.name)
    }

    pub async fn enabled(&self) -> Result<bool> {
        Ok(self.entry_model().await?.enabled)
    }

    pub async fn set_enabled(&self, value: bool) -> Result<()> {
        let mut active = self.entry_model().await?.into_active_model();
        active.enabled = Set(value);
        active.update(self.db.conn()).await?;
        Ok(())
    }

    pub async fn notes(&self) -> Result<String> {
        Ok(self.entry_model().await?.notes)
    }

    pub async fn parent(&self) -> Result<Profile> {
        let profile_id = self.entry_model().await?.profile_id;
        Profile::load(profile_id, self.db.clone(), self.cfg.clone()).await
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, profile: &Profile, mod_: Mod) -> Result<Self> {
        let profile_id = profile.id;
        let mod_id = mod_.id;
        let next_position = profile.mod_entries().await?.len() as i64;

        let model = ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            profile_id: Set(profile_id),
            mod_id: Set(mod_id),
            position: Set(next_position),
            enabled: Set(true),
            notes: Set(String::new()),
        };
        let inserted = model.insert(db.conn()).await?;

        ModEntry::load(inserted.id, mod_id, db.clone(), cfg.clone()).await
    }

    pub async fn remove(self) -> Result<()> {
        let entry_model = self.entry_model().await?;
        let removed_position = entry_model.position;
        let profile_id = entry_model.profile_id;

        self.entry_model().await?.delete(self.db.conn()).await?;

        let trailing = Entity::find()
            .filter(schema::COLUMN.profile_id.eq(profile_id))
            .filter(schema::COLUMN.position.gt(removed_position))
            .order_by_asc(schema::COLUMN.position)
            .all(self.db.conn())
            .await?;

        for model in trailing {
            let mut active = model.into_active_model();
            active.position = Set(active.position.unwrap() - 1);
            active.update(self.db.conn()).await?;
        }

        Ok(())
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, profile: &Profile) -> Result<Vec<Self>> {
        let models = Entity::find()
            .filter(schema::COLUMN.profile_id.eq(profile.id))
            .order_by_asc(schema::COLUMN.position)
            .order_by_asc(schema::COLUMN.id)
            .all(db.conn())
            .await?;

        let mut out = Vec::with_capacity(models.len());
        for model in models {
            out.push(ModEntry::load(model.id, model.mod_id, db.clone(), cfg.clone()).await?);
        }
        Ok(out)
    }
}

impl PartialEq for ModEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entry_id == other.entry_id && self.mod_id == other.mod_id
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

        let mod_entries: Vec<_> = {
            let mut entries = Vec::new();
            for i in 1..=6 {
                let mod_ = game.add_mod(&format!("Mod{i}"), None).await.unwrap();
                entries.push(profile.add_mod_entry(mod_).await.unwrap());
            }
            entries
        };

        assert_eq!(profile.mod_entries().await.unwrap().len(), 6);

        let first = mod_entries.first().unwrap();
        first.clone().remove().await.unwrap();
        assert!(!profile.mod_entries().await.unwrap().contains(first));

        let middle = mod_entries.get(3).unwrap();
        middle.clone().remove().await.unwrap();
        assert!(!profile.mod_entries().await.unwrap().contains(middle));

        let last = mod_entries.get(5).unwrap();
        last.clone().remove().await.unwrap();
        assert!(!profile.mod_entries().await.unwrap().contains(last));

        let remaining: Vec<&ModEntry> = mod_entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| match i {
                0 | 3 | 5 => None,
                _ => Some(entry),
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
