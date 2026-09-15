use super::*;

impl ModEntry {
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

    /// Remove the given [`ModEntry`] from the list.
    pub async fn remove(self) -> Result<(), RemoveError> {
        let mod_name = self
            .mod_()
            .await
            .map_err(RemoveError::Mod)?
            .name()
            .await
            .map_err(RemoveError::ModName)?;

        let profile_name = self
            .parent()
            .await
            .map_err(RemoveError::Profile)?
            .name()
            .await
            .map_err(RemoveError::ProfileName)?;

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
