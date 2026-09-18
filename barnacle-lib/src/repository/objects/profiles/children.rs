use crate::{
    deployers::{self, ItemGroupSpec},
    repository::{
        item_entries::{ItemEntry, ItemEntryGroup},
        items::Item,
    },
};

use super::*;

impl Profile {
    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry, mod_entries::AddError> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>, mod_entries::ListError> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }

    pub(crate) async fn add_item_entry(&self, item: &Item) {
        ItemEntry::add(&self.db, self, item).await
    }

    pub(crate) async fn item_entries(&self, group: impl ItemGroupSpec) -> Vec<ItemEntry> {
        ItemEntry::list_by_group(&self.db, &self.cfg, self, group.key()).await
    }

    pub async fn item_entry_groups(&self) -> Vec<ItemEntryGroup> {
        let game = self.parent().await.unwrap();
        let deploy_kind = game.deploy_kind().await.unwrap();

        let groups = deployers::item_group_defs(deploy_kind);

        ItemEntry::list_groups(&self.db, &self.cfg, self, &groups).await
    }
}
