use barnacle_lib::{DeployKind, ProfileMod, State};
use std::rc::Rc;
use slint::{ModelRc, StandardListViewItem};

slint::include_modules!();

type TableRow = ModelRc<StandardListViewItem>;
type TableModel = ModelRc<TableRow>;

/// Run the GUI
#[tokio::main]
pub async fn main() {
    let app = App::new().unwrap();
    let library_manager = LibraryManager::new().unwrap();

    app.on_open_library_manager(move || {
        let library_manager = library_manager.as_weak();
        library_manager.unwrap().show().unwrap();
    });

    let mut state = State::new().unwrap();

    let game_id = state
        .add_game("Skyrim", DeployKind::Gamebryo)
        .await
        .unwrap();
    let profile_id = state.add_profile(game_id, "Enderal").await.unwrap();
    state.set_current_profile(profile_id).await.unwrap();

    let current_profile = state.current_profile().await.unwrap().unwrap();
    // Get mods from current profile and build model from them
    let mods = state.mods(current_profile).await.unwrap();

    app.global::<ModTableData>()
        .set_model(build_table_model(&mods));

    app.run().unwrap();
}

fn build_table_model(profile_mods: &[ProfileMod]) -> TableModel {
    let mut rows = Vec::new();

    for profile_mod in profile_mods {
        let row = [
            StandardListViewItem::from(if profile_mod.entry().enabled() {
                "✅"
            } else {
                "❌"
            }),
            StandardListViewItem::from(profile_mod.data().name()),
        ];

        rows.push(TableRow::from(row));
    }

    TableModel::from(rows.as_slice())
}
