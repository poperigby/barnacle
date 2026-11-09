use std::env;
use std::path::PathBuf;

use bindgen::callbacks::{IntKind, ParseCallbacks};

#[derive(Debug)]
struct ArchiveCallbacks;

impl ParseCallbacks for ArchiveCallbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        match name {
            // We want these to be i32, because functions that return errno return i32
            "ARCHIVE_OK" | "ARCHIVE_EOF" | "ARCHIVE_RETRY" | "ARCHIVE_WARN" | "ARCHIVE_FAILED"
            | "ARCHIVE_FATAL" => Some(IntKind::Int), // i32
            _ => None, // let bindgen pick
        }
    }
}

fn main() {
    println!("cargo:rustc-link-search=archive");
    println!("cargo:rustc-link-lib=archive");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Needed to evaluate macros bindgen can't parse on its own, like the AE_IF* casts
        .clang_macro_fallback()
        // Only generate libarchive's items. Otherwise the macro fallback also evaluates glibc's
        // self-referential macros (e.g. `#define _PC_LINK_MAX _PC_LINK_MAX`), which duplicates
        // constants already generated from their enums.
        .allowlist_function("archive_.*")
        .allowlist_type("archive.*")
        .allowlist_var("ARCHIVE_.*|AE_.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ArchiveCallbacks))
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
