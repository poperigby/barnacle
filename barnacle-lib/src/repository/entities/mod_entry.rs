use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, QueryOrder,
};

use crate::repository::{
    Cfg, Mod, Profile,
    db::{
        Db,
        models::{mod_entries, mods},
    },
    entities::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct ModEntry {
    pub(crate) entry_id: i64,
    pub(crate) mod_id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl ModEntry {
    pub(crate) fn load(entry_row_id: i64, mod_row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let entry_model = db.run(mod_entries::Entity::find_by_id(entry_row_id).one(db.conn()))?;
        let mod_model = db.run(mods::Entity::find_by_id(mod_row_id).one(db.conn()))?;

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

    fn entry_model(&self) -> Result<mod_entries::Model> {
        let model = self
            .db
            .run(mod_entries::Entity::find_by_id(self.entry_id).one(self.db.conn()))?;
        model.ok_or(Error::RemovedEntity)
    }

    fn mod_model(&self) -> Result<mods::Model> {
        let model = self
            .db
            .run(mods::Entity::find_by_id(self.mod_id).one(self.db.conn()))?;
        model.ok_or(Error::RemovedEntity)
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.mod_model()?.name)
    }

    pub fn enabled(&self) -> Result<bool> {
        Ok(self.entry_model()?.enabled)
    }

    pub fn set_enabled(&self, value: bool) -> Result<()> {
        let mut active = self.entry_model()?.into_active_model();
        active.enabled = Set(value);
        self.db.run(active.update(self.db.conn()))?;
        Ok(())
    }

    pub fn notes(&self) -> Result<String> {
        Ok(self.entry_model()?.notes)
    }

    pub fn parent(&self) -> Result<Profile> {
        let profile_id = self.entry_model()?.profile_id;
        Profile::load(profile_id, self.db.clone(), self.cfg.clone())
    }

    pub(crate) fn add(db: &Db, cfg: &Cfg, profile: &Profile, mod_: Mod) -> Result<Self> {
        let profile_id = profile.id;
        let mod_id = mod_.id;
        let next_position = profile.mod_entries()?.len() as i64;

        let inserted = db.run(async {
            let model = mod_entries::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                profile_id: Set(profile_id),
                mod_id: Set(mod_id),
                position: Set(next_position),
                enabled: Set(true),
                notes: Set(String::new()),
            };
            model.insert(db.conn()).await
        })?;

        ModEntry::load(inserted.id, mod_id, db.clone(), cfg.clone())
    }

    pub fn remove(self) -> Result<()> {
        let entry_model = self.entry_model()?;
        let removed_position = entry_model.position;
        let profile_id = entry_model.profile_id;
        let row_id = entry_model.id;

        self.db.run(async {
            let Some(model) = mod_entries::Entity::find_by_id(row_id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing mod entry during delete".into()));
            };
            model.delete(self.db.conn()).await?;

            let trailing = mod_entries::Entity::find()
                .filter(mod_entries::Column::ProfileId.eq(profile_id))
                .filter(mod_entries::Column::Position.gt(removed_position))
                .order_by_asc(mod_entries::Column::Position)
                .all(self.db.conn())
                .await?;

            for model in trailing {
                let mut active = model.into_active_model();
                active.position = Set(active.position.unwrap() - 1);
                active.update(self.db.conn()).await?;
            }

            Ok(())
        })?;

        Ok(())
    }

    pub(crate) fn list(db: &Db, cfg: &Cfg, profile: &Profile) -> Result<Vec<Self>> {
        let models = db.run(
            mod_entries::Entity::find()
                .filter(mod_entries::Column::ProfileId.eq(profile.id))
                .order_by_asc(mod_entries::Column::Position)
                .order_by_asc(mod_entries::Column::Id)
                .all(db.conn()),
        )?;

        models
            .into_iter()
            .map(|model| ModEntry::load(model.id, model.mod_id, db.clone(), cfg.clone()))
            .collect()
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

    #[test]
    fn test_add() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let profile = game.add_profile("Test").unwrap();

        let mod1 = game.add_mod("Super Duper Mod", None).unwrap();
        let mod2 = game.add_mod("Super Duper Mod: 2", None).unwrap();

        profile.add_mod_entry(mod1).unwrap();
        profile.add_mod_entry(mod2).unwrap();

        assert_eq!(profile.mod_entries().unwrap().len(), 2);
    }

    #[test]
    fn test_remove() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let profile = game.add_profile("Test").unwrap();

        let mod_entries: Vec<_> = (1..=6)
            .map(|i| {
                let mod_ = game.add_mod(&format!("Mod{i}"), None).unwrap();
                profile.add_mod_entry(mod_).unwrap()
            })
            .collect();

        assert_eq!(profile.mod_entries().unwrap().len(), 6);

        let remove_and_check = |entry: &ModEntry| {
            entry.clone().remove().unwrap();
            let entries = profile.mod_entries().unwrap();
            assert!(!entries.contains(entry));
        };

        remove_and_check(mod_entries.first().unwrap());
        remove_and_check(mod_entries.get(3).unwrap());
        remove_and_check(mod_entries.get(5).unwrap());

        let remaining: Vec<&ModEntry> = mod_entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| match i {
                0 | 3 | 5 => None,
                _ => Some(entry),
            })
            .collect();
        assert_eq!(
            profile.mod_entries().unwrap().iter().collect::<Vec<_>>(),
            remaining
        );
    }

    #[test]
    fn test_parent() {
        let repo = Repository::mock();

        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let profile = game.add_profile("The Best Profile").unwrap();
        let mod_ = game
            .add_mod(
                "Better Khajiit Balls 16K - Remastered - 2025 Edition - REAL",
                None,
            )
            .unwrap();
        let entry = profile.add_mod_entry(mod_).unwrap();

        assert_eq!(entry.parent().unwrap(), profile);
    }

    #[test]
    fn test_name() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let profile = game.add_profile("Test").unwrap();
        let mod_ = game.add_mod("Super Duper Mod", None).unwrap();

        profile.add_mod_entry(mod_).unwrap().name().unwrap();
    }

    #[test]
    fn test_enabled() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let profile = game.add_profile("Test").unwrap();
        let mod_ = game.add_mod("Super Duper Mod", None).unwrap();

        let entry = profile.add_mod_entry(mod_).unwrap();

        assert!(entry.enabled().unwrap());

        entry.set_enabled(false).unwrap();

        assert!(!entry.enabled().unwrap());
    }
}
