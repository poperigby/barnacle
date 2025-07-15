use compress_tools::*;

use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let xdg_dirs = xdg::BaseDirectories::with_prefix("barnacle");
    let store_path = xdg_dirs
        .get_data_home()
        .expect("Cannot find HOME")
        .join("store");

    let mod_path = PathBuf::from(&args[1]);
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
