pub mod openmw;

#[derive(Debug, Clone)]
pub struct DiscoveredItem<G>
where
    G: ItemGroupSpec,
{
    pub group: G,
    pub key: String,
    pub relative_path: String,
}

pub trait ItemGroupSpec: Copy + Send {
    fn key(self) -> &'static str;
    fn name(self) -> &'static str;
    fn ordered(self) -> bool;
}

pub trait Deployer {
    fn item_groups() -> Vec<impl ItemGroupSpec>;
}
