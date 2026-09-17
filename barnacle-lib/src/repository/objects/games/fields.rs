use super::*;

impl Game {
    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<(), SetNameError> {
        let conn = self.db.conn();

        let new_name = new_name.to_string();

        let mut active_model = self.active_model(conn).await.map_err(SetNameError::Load)?;
        active_model.name.set_if_not_equals(new_name.to_string());
        active_model.update(conn).await.map_err(|source| {
            if is_unique_violation(&source) {
                SetNameError::Duplicate { name: new_name }
            } else {
                SetNameError::Update(source)
            }
        })?;

        Ok(())
    }

    pub async fn deploy_kind(&self) -> Result<DeployKind, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("deploy_kind", source))?
            .deploy_kind)
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        Ok(Self::dir_from_id(&self.cfg, self.id))
    }

    /// Directory where Barnacle generated files are stored. The directory is
    /// created if it doesn't already exist.
    pub async fn generated_dir(&self) -> Result<PathBuf, GeneratedDirError> {
        let path = state_dir().join("games").join(self.id.to_string());

        create_dir_all(&path).map_err(GeneratedDirError::Create)?;

        Ok(path)
    }

    pub(super) fn dir_from_id(cfg: &Cfg, id: i32) -> PathBuf {
        let library_dir = cfg.read().library_dir().to_path_buf();

        library_dir.join(id.to_string())
    }
}
