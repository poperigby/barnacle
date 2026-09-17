use super::*;

impl ModEntry {
    pub async fn name(&self) -> Result<String, NameError> {
        self.mod_()
            .await
            .map_err(NameError::Load)?
            .name()
            .await
            .map_err(NameError::Name)
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        self.mod_()
            .await
            .map_err(DirError::Load)?
            .dir()
            .await
            .map_err(DirError::Dir)
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
}
