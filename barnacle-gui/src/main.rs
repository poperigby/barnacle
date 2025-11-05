use barnacle_lib::{DeployKind, ProfileMod, state::State};
use slint::{ModelRc, SharedString, StandardListViewItem, spawn_local};

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

    spawn_local({
        let app_weak = app.as_weak();
        let library_manager_weak = library_manager.as_weak();

        app.show().unwrap();

        async move {
            let mut state = State::new().unwrap();

            // Example initialization
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

            // Setup library manager data
            if let Some(library_manager) = library_manager_weak.upgrade() {
                let library_manager_state = LibraryManagerState::new(&state).await;
                let data = library_manager.global::<LibraryManagerData>();
                data.set_games(library_manager_state.games);
                data.set_profiles(library_manager_state.profiles);
                data.set_mods(library_manager_state.mods);
                data.set_tools(library_manager_state.tools);

                library_manager.show().unwrap();
            }
        }
    })
    .unwrap();

    slint::run_event_loop_until_quit().unwrap();
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
