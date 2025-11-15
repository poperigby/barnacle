use iced::{Length::Shrink, advanced::svg, widget::Svg};
use include_dir::{Dir, include_dir};

static ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/icons");

pub fn icon(name: &str) -> Svg<'_> {
    let bytes = ICONS
        .get_file(format!("{name}.svg"))
        .expect("Missing icon")
        .contents();

    let handle = svg::Handle::from_memory(bytes);
    Svg::new(handle).width(Shrink)
}
