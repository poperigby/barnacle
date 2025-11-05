use barnacle_lib::{Game, GameId, state::State};
use slint::{ModelRc, SharedString, StandardListViewItem};

use crate::{ListModel, TableModel, TableRow};

/// Represents the currently loaded view of the library manager
#[derive(Debug)]
pub struct LibraryManagerState {
    games: Vec<Game>,
    pub games_model: ListModel,
    pub current_game: Option<GameId>,
    pub profiles_model: TableModel,
    pub mods_model: TableModel,
    pub tools_model: TableModel,
}

impl LibraryManagerState {
    pub async fn new(state: State) -> Self {
        let games = state.games().await.unwrap();

        let games_model = ModelRc::from(
            games
                .iter()
                .map(|g| g.name().into())
                .collect::<Vec<SharedString>>()
                .as_slice(),
        );

        let current_game = games.first().and_then(|g| g.id());

        let mut library_manager_state = Self {
            games,
            games_model,
            current_game,
            profiles_model: TableModel::from(&[] as &[TableRow]),
            mods_model: TableModel::from(&[] as &[TableRow]),
            tools_model: TableModel::from(&[] as &[TableRow]),
        };

        // If we have a selected game, populate the related data
        if let Some(id) = current_game {
            library_manager_state.load(state, id).await;
        }

        library_manager_state
    }

    pub async fn select_game(&mut self, state: State, index: usize) {
        if let Some(game) = self.games.get(index) {
            self.load(state, game.id().unwrap()).await;
        }
    }

    /// Loads the library manager state for the selected game
    async fn load(&mut self, state: State, game_id: GameId) {
        self.current_game = Some(game_id);

        self.profiles_model = TableModel::from(
            state
                .profiles(game_id)
                .await
                .unwrap()
                .iter()
                .map(|profile| TableRow::from([StandardListViewItem::from(profile.name())]))
                .collect::<Vec<_>>()
                .as_slice(),
        );

        self.mods_model = TableModel::from(
            state
                .mods(game_id)
                .await
                .unwrap()
                .iter()
                .map(|mod_| TableRow::from([StandardListViewItem::from(mod_.name())]))
                .collect::<Vec<_>>()
                .as_slice(),
        );

        self.tools_model = TableModel::from(
            state
                .tools(game_id)
                .await
                .unwrap()
                .iter()
                .map(|tool| TableRow::from([StandardListViewItem::from(tool.name())]))
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }
}
