use barnacle_lib::{Game, GameId, state::State};
use slint::StandardListViewItem;

use crate::{TableModel, TableRow};

/// Represents the currently loaded view of the library manager
#[derive(Debug)]
pub struct LibraryManagerState {
    pub games: Vec<Game>,
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
            games,
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
