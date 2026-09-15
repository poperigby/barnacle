use super::*;

impl Profile {
    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry, mod_entry::AddError> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>, mod_entry::ListError> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }
}
