use barnacle_lib::{Game, GameId, Mod, Profile, Tool, state::State};

/// Represents the currently loaded view of the library manager
#[derive(Debug)]
pub struct LibraryManagerState {
    pub games: Vec<Game>,
    pub current_game: Option<GameId>,
    pub profiles: Vec<Profile>,
    pub mods: Vec<Mod>,
    pub tools: Vec<Tool>,
}

impl LibraryManagerState {
    pub async fn load(state: &State) -> Self {
        let games = state.games().await.unwrap();
        let current_game = games.first().and_then(|g| g.id()).unwrap();

        Self {
            games,
            current_game: Some(current_game),
            profiles: state.profiles(current_game).await.unwrap(),
            mods: state.mods(current_game).await.unwrap(),
            tools: Vec::new(),
        }
    }
}
