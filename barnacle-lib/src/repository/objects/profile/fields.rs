use super::*;

impl Profile {
    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<(), SetNameError> {
        let mut active_model = self
            .active_model(self.db.conn())
            .await
            .map_err(SetNameError::Load)?;

        active_model.name.set_if_not_equals(new_name.to_string());
        active_model
            .update(self.db.conn())
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    SetNameError::Duplicate {
                        name: new_name.to_string(),
                    }
                } else {
                    SetNameError::Update(source)
                }
            })?;

        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        Ok(self
            .parent()
            .await
            .map_err(DirError::Parent)?
            .dir()
            .await
            .map_err(DirError::ParentDir)?
            .join("profiles")
            .join(self.id.to_string()))
    }

    /// Directory where Barnacle generated files are stored. The directory is
    /// created if it doesn't already exist.
    pub async fn generated_dir(&self) -> Result<PathBuf, GeneratedDirError> {
        let parent_path = self
            .parent()
            .await
            .map_err(GeneratedDirError::Parent)?
            .generated_dir()
            .await
            .map_err(GeneratedDirError::ParentGeneratedDir)?;

        let path = parent_path.join("profiles").join(self.id.to_string());

        create_dir_all(&path).map_err(GeneratedDirError::Create)?;

        Ok(path)
    }

    /// Returns the parent [`Game`] of this [`Profile`].
    pub async fn parent(&self) -> Result<Game, ParentError> {
        let parent_game_id = self
            .model(self.db.conn())
            .await
            .map_err(ParentError::Load)?
            .game_id;

        Ok(Game::from_id(parent_game_id, &self.db, &self.cfg))
    }
}
