use super::*;

impl Game {
    pub(crate) async fn active_profile_id(&self) -> Result<Option<i32>, LoadModelError> {
        Ok(self.model(self.db.conn()).await?.active_profile_id)
    }

    pub(crate) async fn set_active_profile_id(
        &self,
        profile_id: Option<i32>,
    ) -> Result<(), SetActiveProfileIdError> {
        let conn = self.db.conn();

        let mut active_model = self
            .active_model(conn)
            .await
            .map_err(SetActiveProfileIdError::Load)?;

        active_model.active_profile_id.set_if_not_equals(profile_id);

        active_model
            .update(conn)
            .await
            .map_err(SetActiveProfileIdError::Update)?;

        Ok(())
    }

    pub async fn active_profile(&self) -> Result<Option<Profile>, profile::ActiveError> {
        Profile::active(&self.db, &self.cfg, self).await
    }

    pub async fn search_profile(
        &self,
        name: &str,
    ) -> Result<Option<Profile>, profile::SearchError> {
        Profile::search(self.db.clone(), self.cfg.clone(), self, name).await
    }

    pub async fn add_profile(&self, name: &str) -> Result<Profile, profile::AddError> {
        Profile::add(&self.db, &self.cfg, self, name).await
    }

    pub async fn profiles(&self) -> Result<Vec<Profile>, profile::ListError> {
        Profile::list(&self.db, &self.cfg, self).await
    }

    pub async fn add_target(&self, name: &str, path: &Path) -> Result<Target, target::AddError> {
        Target::add(&self.db, &self.cfg, self, name, path).await
    }

    pub async fn targets(&self) -> Result<Vec<Target>, target::ListError> {
        Target::list(&self.db, &self.cfg, self).await
    }

    pub fn new_mod(&self, name: &str) -> NewMod {
        Mod::new_mod(self.db.clone(), self.cfg.clone(), self, name)
    }

    pub async fn mods(&self) -> Result<Vec<Mod>, mod_::ListError> {
        Mod::list(&self.db.clone(), &self.cfg.clone(), self).await
    }
}
