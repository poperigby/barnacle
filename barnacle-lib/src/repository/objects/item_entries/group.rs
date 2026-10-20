use crate::{
    Profile,
    integrations::openmw::{self, ItemKind as OpenMwKind},
    repository::{
        item_entries::ItemEntry, items::Kind, objects::item_entries::error::AutoSortError,
    },
};

#[derive(Debug, Clone)]
pub struct ItemGroup {
    pub(crate) profile: Profile,
    pub(crate) kind: Kind,
    pub(crate) entries: Vec<ItemEntry>,
}

impl ItemGroup {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn entries(&self) -> &[ItemEntry] {
        &self.entries
    }

    pub fn supports_reordering(&self) -> bool {
        self.kind.supports_reordering()
    }

    pub fn supports_auto_sort(&self) -> bool {
        self.kind.supports_auto_sort()
    }

    pub async fn auto_sort(&self) -> Result<(), AutoSortError> {
        if !self.supports_auto_sort() {
            return Err(AutoSortError::NotSupported(self.kind));
        }

        match self.kind {
            Kind::OpenMW(OpenMwKind::Plugin) => openmw::OpenMw::sort_plugins(&self.profile).await,
            _ => unreachable!(),
        };

        Ok(())
    }
}

impl PartialEq for ItemGroup {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
