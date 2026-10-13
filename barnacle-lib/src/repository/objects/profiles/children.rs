use crate::{
    integrations::{self},
    repository::{
        item_entries::{ItemEntry, ItemGroup},
        items::{Item, kind::Kind},
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

    /// A list of all [`ItemEntry`]s of a particular kind. Even if multiple [`Mod`]s provide
    /// conflicting [`Item`]s, this list will only return one entry per relative path.
    /// The returned entry will be the one owned by whatever [`ModEntry`] won the conflict.
    pub(crate) async fn item_entries(&self, kind: Kind) -> Vec<ItemEntry> {
        ItemEntry::list_by_kind(&self.db, &self.cfg, self, kind).await
    }

    pub async fn item_groups(&self) -> Vec<ItemGroup> {
        let game = self.parent().await.unwrap();
        let deploy_kind = game.kind().await.unwrap();

        let groups = integrations::item_kinds(&deploy_kind);

        ItemEntry::list_groups(&self.db, &self.cfg, self, &groups).await
    }
}
