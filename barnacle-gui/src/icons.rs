use std::collections::HashMap;

use iced::{
    Theme,
    advanced::svg::Handle,
    widget::{Svg, svg::Style},
};
use include_dir::{Dir, include_dir};
use once_cell::sync::Lazy;

static ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/icons");
static HANDLES: Lazy<HashMap<&'static str, Handle>> = Lazy::new(|| {
    ICONS
        .files()
        .filter_map(|file| {
            let name = file.path().file_stem()?.to_str()?;
            Some((name, Handle::from_memory(file.contents())))
        })
        .collect()
});

pub fn icon(name: &str) -> Svg<'_> {
    let handle = HANDLES.get(name).expect("Failed to find icon");

    Svg::new(handle.clone())
        .width(24)
        .height(24)
        .style(|theme: &Theme, _| Style {
            color: Some(theme.palette().text),
        })
}
