use std::fs::remove_dir_all;

use barnacle_lib::{DeployKind, ProfileMod, fs::data_dir, state::State};
use slint::{ModelRc, SharedString, StandardListViewItem, run_event_loop, spawn_local};

use crate::library_manager::LibraryManagerState;

mod library_manager;

slint::include_modules!();

type TableRow = ModelRc<StandardListViewItem>;
type TableModel = ModelRc<TableRow>;
type ListModel = ModelRc<SharedString>;

/// Run the GUI
pub fn main() {
    let app = App::new().unwrap();
    let library_manager = LibraryManager::new().unwrap();

    let mut state = State::new().unwrap();

    app.on_open_library_manager({
        let library_manager_weak = library_manager.as_weak().unwrap();

        move || {
            library_manager_weak.show().unwrap();
        }
    });

    library_manager.on_game_changed({
        let global = library_manager.global::<LibraryManagerData>();

        move |game_index| {
            spawn_local(async move {
                println!("{}", game_index);
            })
            .unwrap();
        }
    });

    spawn_local({
        let app_weak = app.as_weak();
        let lm_weak = library_manager.as_weak();

        async move {
            let library_manager_state = LibraryManagerState::new(state.clone()).await;

            state
                .add_game("Morrowind", DeployKind::OpenMW)
                .await
                .unwrap();
            let game_id = state
                .add_game("Skyrim", DeployKind::Gamebryo)
                .await
                .unwrap();
            let profile_id = state.add_profile(game_id, "Enderal").await.unwrap();
            state.set_current_profile(profile_id).await.unwrap();

            let current_profile = state.current_profile().await.unwrap().unwrap();
            state.add_mod(game_id, None, "Test").await.unwrap();
            let mods = state.profile_mods(current_profile).await.unwrap();

            // Update mod table
            if let Some(app) = app_weak.upgrade() {
                app.global::<ModTableData>()
                    .set_model(build_mod_table_model(&mods));
            }
        }
    })
    .unwrap();

    app.show().unwrap();
    // library_manager.show().unwrap();

    run_event_loop().unwrap();

    // TODO: For testing
    remove_dir_all(data_dir()).unwrap();
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

fn load_library_manager_data(state: LibraryManagerState, global: LibraryManagerData) {
    global.set_games(state.games_model);
    global.set_profiles(state.profiles_model);
    global.set_mods(state.mods_model);
    global.set_tools(state.tools_model);
}
