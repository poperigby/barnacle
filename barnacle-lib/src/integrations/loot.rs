use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::PathBuf,
};

use crate::{fs::cache_dir, repository::item_entries::ItemEntry};

const MASTERLIST_VERSION: &str = "0.29";

#[derive(Debug, Clone)]
pub(crate) struct Plugin {
    pub(crate) item: ItemEntry,
    pub(crate) absolute_path: PathBuf,
}

pub fn masterlist(kind: libloot::GameType) -> PathBuf {
    let path = masterlist_path(kind);

    if !path.exists() {
        fetch_masterlist(kind);
    }

    path
}

fn masterlist_path(kind: libloot::GameType) -> PathBuf {
    loot_dir(kind).join("masterlist.yaml")
}

fn fetch_masterlist(kind: libloot::GameType) {
    let slug = kind_slug(kind);
    let url = format!(
        "https://raw.githubusercontent.com/loot/{slug}/refs/heads/v{MASTERLIST_VERSION}/masterlist.yaml",
    );

    let bytes = ureq::get(url)
        .call()
        .unwrap()
        .body_mut()
        .read_to_vec()
        .unwrap();

    let mut file = File::create(masterlist_path(kind)).unwrap();
    file.write_all(&bytes).unwrap();
}

fn loot_dir(kind: libloot::GameType) -> PathBuf {
    let slug = kind_slug(kind);
    let path = cache_dir().join("loot").join(slug);

    create_dir_all(&path).unwrap();

    path
}

fn kind_slug(kind: libloot::GameType) -> String {
    use libloot::GameType::*;
    match kind {
        Morrowind | OpenMW => "morrowind",
        Oblivion | OblivionRemastered => "oblivion",
        Skyrim => "skyrim",
        SkyrimSE | SkyrimVR => "skyrimse",
        Fallout3 => "fallout3",
        FalloutNV => "falloutnv",
        Fallout4 | Fallout4VR => "fallout4",
        Starfield => "starfield",
        _ => unimplemented!(),
    }
    .to_string()
}
