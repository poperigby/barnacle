pub mod openmw;

#[derive(Debug, Clone)]
pub struct ParsedItem<G>
where
    G: ItemGroupSpec,
{
    pub name: String,
    pub relative_path: String,
    pub group: G,
}

pub trait ItemGroupSpec: Copy + Send {
    fn key(self) -> &'static str;
    fn name(self) -> &'static str;
    fn ordered(self) -> bool;
}

pub trait Deployer {
    fn item_groups() -> Vec<impl ItemGroupSpec>;
}
