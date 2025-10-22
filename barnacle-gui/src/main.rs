use barnacle_lib::State;

slint::include_modules!();

/// Run the GUI
pub fn main() {
    let app = App::new().unwrap();

    let state = State::new().unwrap();
    let current_profile = state.current_profile();
    // Get mods from current profile and build model from them

    app.run().unwrap();
}
