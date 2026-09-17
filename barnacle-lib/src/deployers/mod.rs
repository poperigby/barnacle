pub mod openmw;

#[derive(Debug, Clone)]
pub struct ParsedItem<G>
where
    G: ItemGroup,
{
    pub name: String,
    pub relative_path: String,
    pub group: G,
}

pub trait ItemGroup: Copy + Send {
    fn key(self) -> &'static str;
    fn name(self) -> &'static str;
    fn sortable(self) -> bool;
}
