use crate::{deployers::ItemGroupSpec, repository::objects::ItemEntry};

use super::*;

impl Profile {
    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry, mod_entries::AddError> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>, mod_entries::ListError> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }

    pub async fn item_entries(&self, group: impl ItemGroupSpec) -> Vec<ItemEntry> {
        ItemEntry::list(&self.db, &self.cfg, self, group).await
    }
}
