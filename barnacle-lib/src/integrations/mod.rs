use std::fmt::Debug;

use crate::repository::{games, items};

use strum::IntoEnumIterator;

pub mod openmw;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct FoundItem {
    pub kind: items::Kind,
    pub key: String,
    pub relative_path: String,
}

pub(crate) fn item_kinds(kind: &games::Kind) -> Vec<items::Kind> {
    match kind {
        games::Kind::OpenMW => openmw::ItemKind::iter().map(items::Kind::OpenMW).collect(),
        _ => unimplemented!(),
    }
}
