use crate::DeployKind;

pub mod openmw;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct FoundItem<G>
where
    G: ItemGroupSpec,
{
    pub group: G,
    pub key: String,
    pub relative_path: String,
}

pub(crate) trait ItemGroupSpec: Copy + Send {
    fn key(self) -> &'static str;
    fn name(self) -> &'static str;
    fn ordered(self) -> bool;

    fn definition(self) -> ItemGroupDef {
        ItemGroupDef {
            key: self.key().to_string(),
            name: self.name().to_string(),
            ordered: self.ordered(),
        }
    }
}

pub(crate) fn item_group_defs(deploy_kind: DeployKind) -> Vec<ItemGroupDef> {
    match deploy_kind {
        DeployKind::OpenMW => openmw::OpenMw::item_groups()
            .into_iter()
            .map(ItemGroupSpec::definition)
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) struct ItemGroupDef {
    pub key: String,
    pub name: String,
    pub ordered: bool,
}

pub(crate) trait Deployer {
    fn item_groups() -> Vec<impl ItemGroupSpec>;
}
