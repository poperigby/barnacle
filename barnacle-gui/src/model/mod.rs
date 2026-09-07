use barnacle_lib::{
    Repository,
    repository::{Game, Mod, ModEntry, Profile, handles::Result},
};
use derive_more::Display;
use iced::widget::combo_box;

use crate::model::mutation::MutationResult;

pub use mutation::Mutation;

mod mutation;

/// Runtime data used to render the loaded GUI.
///
/// Contains the in-memory projection of [`Repository`] state plus derived render
/// adapters. Component `view()` functions can read from this cheaply.
/// [`Repository`] data is updated only from successful [`MutationResult`]s.
#[derive(Debug, Clone)]
pub struct Model {
    active_game: Option<GameRow>,
    games: Vec<GameRow>,

    active_profile: Option<ProfileRow>,
    profile_selector_state: combo_box::State<ProfileRow>,
    profiles: Vec<ProfileRow>,

    mods: Vec<ModRow>,
    mod_entries: Vec<ModEntryRow>,
}

impl Model {
    pub async fn load(repo: &Repository) -> Result<Self> {
        // We barely have anything to load if there's no active game (which means there aren't any games).
        let active_game_handle = match repo.active_game().await? {
            Some(game) => game,
            None => {
                return Ok(Self::new(
                    None,
                    Vec::new(),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ));
            }
        };

        let active_game = GameRow::load(active_game_handle.clone()).await?;
        let games = GameRow::load_all(repo.games().await?).await?;

        let active_profile_handle = match active_game_handle.active_profile().await? {
            Some(profile) => profile,
            None => {
                return Ok(Self::new(
                    Some(active_game),
                    games,
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ));
            }
        };

        let active_profile = ProfileRow::load(active_profile_handle.clone()).await?;
        let profiles = ProfileRow::load_all(active_game_handle.profiles().await?).await?;

        let mods = ModRow::load_all(active_game_handle.mods().await?).await?;
        let mod_entries = ModEntryRow::load_all(active_profile_handle.mod_entries().await?).await?;

        Ok(Self::new(
            Some(active_game),
            games,
            Some(active_profile),
            profiles,
            mods,
            mod_entries,
        ))
    }

    fn new(
        active_game: Option<GameRow>,
        games: Vec<GameRow>,
        active_profile: Option<ProfileRow>,
        profiles: Vec<ProfileRow>,
        mods: Vec<ModRow>,
        mod_entries: Vec<ModEntryRow>,
    ) -> Self {
        Self {
            active_game,
            games,
            active_profile,
            profile_selector_state: combo_box::State::new(profiles.clone()),
            profiles,
            mods,
            mod_entries,
        }
    }

    pub fn apply(result: MutationResult) {
        todo!()
    }

    pub fn active_game(&self) -> &Option<GameRow> {
        &self.active_game
    }

    pub fn games(&self) -> &Vec<GameRow> {
        &self.games
    }

    pub fn active_profile(&self) -> &Option<ProfileRow> {
        &self.active_profile
    }

    pub fn profile_selector_state(&self) -> &combo_box::State<ProfileRow> {
        &self.profile_selector_state
    }

    pub fn profiles(&self) -> &Vec<ProfileRow> {
        &self.profiles
    }

    pub fn mods(&self) -> &Vec<ModRow> {
        &self.mods
    }

    pub fn mod_entries(&self) -> &Vec<ModEntryRow> {
        &self.mod_entries
    }
}

#[derive(Debug, Clone)]
pub struct GameRow {
    handle: Game,
    pub name: String,
}

impl GameRow {
    pub fn handle(&self) -> Game {
        self.handle.clone()
    }

    async fn load(game: Game) -> Result<Self> {
        Ok(Self {
            name: game.name().await?,
            handle: game,
        })
    }

    async fn load_all(games: Vec<Game>) -> Result<Vec<Self>> {
        let mut rows = Vec::with_capacity(games.len());

        for game in games {
            rows.push(Self::load(game).await?);
        }

        Ok(rows)
    }
}

#[derive(Debug, Clone, Display)]
#[display("{name}")]
pub struct ProfileRow {
    handle: Profile,
    pub name: String,
}

impl ProfileRow {
    pub fn handle(&self) -> Profile {
        self.handle.clone()
    }

    async fn load(profile: Profile) -> Result<Self> {
        Ok(Self {
            name: profile.name().await?,
            handle: profile,
        })
    }

    async fn load_all(profiles: Vec<Profile>) -> Result<Vec<Self>> {
        let mut rows = Vec::with_capacity(profiles.len());

        for profile in profiles {
            rows.push(Self::load(profile).await?);
        }

        Ok(rows)
    }
}

#[derive(Debug, Clone)]
pub struct ModRow {
    handle: Mod,
    pub name: String,
}

impl ModRow {
    async fn load(mod_: Mod) -> Result<Self> {
        Ok(Self {
            name: mod_.name().await?,
            handle: mod_,
        })
    }

    async fn load_all(mods: Vec<Mod>) -> Result<Vec<Self>> {
        let mut rows = Vec::with_capacity(mods.len());

        for mod_ in mods {
            rows.push(Self::load(mod_).await?);
        }

        Ok(rows)
    }
}

#[derive(Debug, Clone)]
pub struct ModEntryRow {
    handle: ModEntry,
    pub name: String,
    pub enabled: bool,
}

impl ModEntryRow {
    pub fn handle(&self) -> ModEntry {
        self.handle.clone()
    }

    async fn load(mod_entry: ModEntry) -> Result<Self> {
        Ok(Self {
            name: mod_entry.name().await?,
            enabled: mod_entry.enabled().await?,
            handle: mod_entry,
        })
    }

    async fn load_all(mod_entries: Vec<ModEntry>) -> Result<Vec<Self>> {
        let mut rows = Vec::with_capacity(mod_entries.len());

        for mod_entry in mod_entries {
            rows.push(Self::load(mod_entry).await?);
        }

        Ok(rows)
    }
}
