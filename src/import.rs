use compress_tools::{Ownership, uncompress_archive};
use std::{
    fs::{self, File},
    path::Path,
};

use crate::data_dir;

pub fn import_mod(mod_path: &Path) {
    let store_path = data_dir().join("store");

    let mod_hash = blake3::hash(&fs::read(&mod_path).unwrap());

    let output_dir = store_path.join(format!(
        "{}-{}",
        mod_hash,
        mod_path.file_stem().unwrap().to_str().unwrap()
    ));

    uncompress_archive(
        &mut File::open(&mod_path).unwrap(),
        &output_dir,
        Ownership::Preserve,
    )
    .unwrap();

    println!("{:?}", output_dir);
}
