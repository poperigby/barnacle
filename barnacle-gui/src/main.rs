slint::include_modules!();

/// Run the GUI
pub fn main() {
    let app = App::new().unwrap();

    // Load current profile
    // let current_profile = manager.current_profile();
    // Get mods from current profile and build model from them

    app.run().unwrap();
}
