use barnacle_lib::{DeployKind, ProfileMod, state::State};
use slint::{ModelRc, SharedString, StandardListViewItem};

use crate::library_manager::LibraryManagerState;

mod library_manager;

slint::include_modules!();

type TableRow = ModelRc<StandardListViewItem>;
type TableModel = ModelRc<TableRow>;
type ListModel = ModelRc<SharedString>;

/// Run the GUI
#[tokio::main]
pub async fn main() {
    let app = App::new().unwrap();
    let library_manager = LibraryManager::new().unwrap();

    // app.on_open_library_manager({
    //     let library_manager = library_manager.as_weak();
    //     move || {
    //         library_manager.unwrap().show().unwrap();
    //     }
    // });

    let mut state = State::new().unwrap();

    let game_id = state
        .add_game("Skyrim", DeployKind::Gamebryo)
        .await
        .unwrap();
    let profile_id = state.add_profile(game_id, "Enderal").await.unwrap();
    state.set_current_profile(profile_id).await.unwrap();

    let current_profile = state.current_profile().await.unwrap().unwrap();
    // Get mods from current profile and build model from them
    state.add_mod(game_id, None, "Test").await.unwrap();
    let mods = state.profile_mods(current_profile).await.unwrap();

    app.global::<ModTableData>()
        .set_model(build_mod_table_model(&mods));

    let library_manager_state = LibraryManagerState::load(&state).await;
    let library_manager_data = library_manager.global::<LibraryManagerData>();
    library_manager_data.set_games(library_manager_state.games);
    library_manager_data.set_profiles(library_manager_state.profiles);
    library_manager_data.set_mods(library_manager_state.mods);

    library_manager.show().unwrap();
    app.run().unwrap();
}

fn build_mod_table_model(profile_mods: &[ProfileMod]) -> TableModel {
    TableModel::from(
        profile_mods
            .iter()
            .map(|profile_mod| {
                TableRow::from([
                    StandardListViewItem::from(if profile_mod.entry().enabled() {
                        "✅"
                    } else {
                        "❌"
                    }),
                    StandardListViewItem::from(profile_mod.data().name()),
                ])
            })
            .collect::<Vec<TableRow>>()
            .as_slice(),
    )
}
