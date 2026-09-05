use std::collections::HashMap;
use std::sync::LazyLock;

use iced::{
    Element, Theme,
    advanced::svg::Handle,
    widget::{Svg, svg::Style},
};
use include_dir::{Dir, include_dir};

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

static ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/icons");
static HANDLES: LazyLock<HashMap<&'static str, Handle>> = LazyLock::new(|| {
    ICONS
        .files()
        .filter_map(|file| {
            if file.path().extension().and_then(|ext| ext.to_str()) != Some("svg") {
                return None;
            }

            let name = file.path().file_stem()?.to_str()?;
            Some((name, Handle::from_memory(file.contents())))
        })
        .collect()
});

fn icon(name: impl AsRef<str>) -> Svg<'static> {
    let name = name.as_ref();
    let handle = HANDLES
        .get(name)
        .unwrap_or_else(|| panic!("icon `{name}` must exist"));

    Svg::new(handle.clone())
        .width(24)
        .height(24)
        .style(|theme: &Theme, _| Style {
            color: Some(theme.palette().text),
        })
}

impl<'a, Message> From<Icon> for Element<'a, Message> {
    fn from(value: Icon) -> Self {
        icon(value).into()
    }
}
