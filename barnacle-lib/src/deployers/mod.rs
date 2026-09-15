use std::path::PathBuf;

pub mod openmw;

#[derive(Debug, Clone)]
pub struct ParsedItem<G>
where
    G: ItemGroup,
{
    pub name: String,
    pub path: PathBuf,
    pub group: G,
}

pub trait ItemGroup: Copy + Send {
    fn key(self) -> &'static str;
    fn name(self) -> &'static str;
    fn sortable(self) -> bool;
}
