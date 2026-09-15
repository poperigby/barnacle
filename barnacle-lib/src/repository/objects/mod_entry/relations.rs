use super::*;

impl ModEntry {
    pub(super) async fn mod_(&self) -> Result<Mod, ModError> {
        let id = self
            .model(self.db.conn())
            .await
            .map_err(ModError::LoadEntry)?
            .mod_id;

        Ok(Mod::from_id(id, &self.db, &self.cfg))
    }

    /// Returns the parent [`Profile`] of this [`ModEntry`].
    pub async fn parent(&self) -> Result<Profile, ParentError> {
        Ok(Profile::from_id(
            self.model(self.db.conn())
                .await
                .map_err(ParentError::Load)?
                .profile_id,
            &self.db,
            &self.cfg,
        ))
    }
}
