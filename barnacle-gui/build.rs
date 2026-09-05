use std::{env, fs, io, path::Path};

use heck::ToUpperCamelCase;
use quote::{format_ident, quote};

fn main() -> io::Result<()> {
    generate_icon_enum()?;

    Ok(())
}

fn generate_icon_enum() -> io::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let icons_dir = Path::new(&manifest_dir).join("assets/icons");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    let generated_path = Path::new(&out_dir).join("icons.rs");

    println!("cargo:rerun-if-changed={}", icons_dir.display());

    let mut icons = Vec::new();
    for entry in fs::read_dir(&icons_dir)? {
        let path = entry?.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("icon file name must be valid UTF-8");

        let mut variant = stem.to_upper_camel_case();
        if variant.starts_with(|ch: char| ch.is_ascii_digit()) {
            variant.insert_str(0, "Icon");
        }

        icons.push((format_ident!("{variant}"), stem.to_string()));
    }

    icons.sort_by_key(|i| i.0.to_string());

    let variants = icons.iter().map(|(variant, _)| variant);
    let match_arms = icons
        .iter()
        .map(|(variant, name)| quote! { Self::#variant => #name });

    let generated = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icon {
            #(#variants,)*
        }

        impl Icon {
            pub const fn name(self) -> &'static str {
                match self {
                    #(#match_arms,)*
                }
            }
        }

        impl AsRef<str> for Icon {
            fn as_ref(&self) -> &str {
                self.name()
            }
        }
    };

    fs::write(generated_path, generated.to_string())
}
