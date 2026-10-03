use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use fluent_static_codegen::MessageBundleBuilder;
use walkdir::{DirEntry, WalkDir};

fn main() -> io::Result<()> {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));

    let l10n_dir = manifest_dir.join("l10n");

    println!("cargo:rerun-if-changed={}", l10n_dir.display());

    fs::create_dir_all(&out_dir)?;

    let mut builder = MessageBundleBuilder::new("Translations");

    builder
        .set_default_language("en-US")
        .expect("Default language should be valid language identifier");

    // We need to make sure base translation strings from barnacle-lib load first, because they contain
    // messages that are referenced by other translation files.
    add_resources(&l10n_dir, &mut builder)?;

    builder
        .build()
        .unwrap()
        .write_to_file(out_dir.join("messages.rs"))
        .expect("Output directory should exist and be writeable to save generated code");

    Ok(())
}

fn add_resources(source_dir: &Path, builder: &mut MessageBundleBuilder) -> io::Result<()> {
    for entry in source_dir.read_dir()? {
        let locale_dir = entry?.path();
        let lang_id = locale_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("Locale directory name must be valid UTF-8");

        // Fluent file with basic definitions (e.g. Game) that is referenced by other fluent files.
        // Has to be added first, or fluent-static is upset :(
        let main_file = locale_dir.join("main.ftl");
        builder.add_resource(lang_id, &main_file).unwrap();

        let files: Vec<_> = WalkDir::new(&locale_dir)
            .into_iter()
            .map(Result::unwrap)
            .filter(is_fluent_file)
            .filter(|e| e.path() != main_file)
            .map(|entry| entry.into_path())
            .collect();

        for path in files {
            builder.add_resource(lang_id, &path).unwrap();
        }
    }

    Ok(())
}

fn is_fluent_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "ftl")
}
