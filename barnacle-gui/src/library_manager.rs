use barnacle_lib::{GameId, state::State};
use slint::{ModelRc, SharedString, StandardListViewItem};

use crate::{ListModel, TableModel, TableRow};

/// Represents the currently loaded view of the library manager
#[derive(Debug)]
pub struct LibraryManagerState {
    pub games: ListModel,
    pub current_game: Option<GameId>,
    pub profiles: TableModel,
    pub mods: TableModel,
    // pub tools: TableModel,
}

impl LibraryManagerState {
    pub async fn load(state: &State) -> Self {
        let games = state.games().await.unwrap();
        let current_game = games.first().and_then(|g| g.id()).unwrap();

        Self {
            games: ModelRc::from(
                games
                    .iter()
                    .map(|g| g.name().into())
                    .collect::<Vec<SharedString>>()
                    .as_slice(),
            ),
            current_game: Some(current_game),
            profiles: TableModel::from(
                state
                    .profiles(current_game)
                    .await
                    .unwrap()
                    .iter()
                    .map(|profile| TableRow::from([StandardListViewItem::from(profile.name())]))
                    .collect::<Vec<TableRow>>()
                    .as_slice(),
            ),
            mods: TableModel::from(
                state
                    .mods(current_game)
                    .await
                    .unwrap()
                    .iter()
                    .map(|mod_| TableRow::from([StandardListViewItem::from(mod_.name())]))
                    .collect::<Vec<TableRow>>()
                    .as_slice(),
            ),
        }
    }
}
