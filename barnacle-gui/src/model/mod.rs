use barnacle_lib::{
    Repository,
    repository::{Game, Mod, ModEntry, Profile},
};
use derive_more::Display;
use futures::future::try_join_all;

pub use mutation::Mutation;

mod mutation;

/// Runtime data used to render the loaded GUI.
///
/// Contains the in-memory projection of [`Repository`] state. Component `view()` functions can read from
/// this cheaply. [`Repository`] data is updated only from successful mutations.
#[derive(Debug, Clone)]
pub struct Model {
    active_game: Option<GameItem>,
    games: Vec<GameItem>,

    active_profile: Option<ProfileItem>,

    mods: Vec<ModItem>,
    mod_entries: Vec<ModEntryItem>,
}

impl Model {
    pub async fn load(repo: &Repository) -> anyhow::Result<Self> {
        // We barely have anything to load if there's no active game (which means there aren't any games).
        let active_game_handle = match repo.active_game().await? {
            Some(game) => game,
            None => {
                return Ok(Self::new(None, Vec::new(), None, Vec::new(), Vec::new()));
            }
        };

        let active_game = GameItem::load_active(repo).await?;

        let games = GameItem::load_all(repo).await?;

        let active_profile_handle = match active_game_handle.active_profile().await? {
            Some(profile) => profile,
            None => {
                return Ok(Self::new(active_game, games, None, Vec::new(), Vec::new()));
            }
        };

        let active_profile = ProfileItem::load(active_profile_handle.clone()).await?;

        let mods = ModItem::load_all(&active_game_handle).await?;
        let mod_entries = ModEntryItem::load_all(&active_profile_handle).await?;

        Ok(Self::new(
            active_game,
            games,
            Some(active_profile),
            mods,
            mod_entries,
        ))
    }

    fn new(
        active_game: Option<GameItem>,
        games: Vec<GameItem>,
        active_profile: Option<ProfileItem>,
        mods: Vec<ModItem>,
        mod_entries: Vec<ModEntryItem>,
    ) -> Self {
        Self {
            active_game,
            games,
            active_profile,
            mods,
            mod_entries,
        }
    }

    pub fn active_game(&self) -> Option<GameItem> {
        self.active_game.clone()
    }

    pub fn games(&self) -> Vec<GameItem> {
        self.games.clone()
    }

    pub fn game(&self, game: &Game) -> Option<GameItem> {
        self.games()
            .iter()
            .find(|item| item.handle() == *game)
            .cloned()
    }

    pub fn active_profile(&self) -> Option<ProfileItem> {
        self.active_profile.clone()
    }

    pub fn profile(&self, profile: &Profile) -> Option<ProfileItem> {
        self.active_game()?
            .profiles
            .iter()
            .find(|item| item.handle() == *profile)
            .cloned()
    }

    pub fn mods(&self) -> Vec<ModItem> {
        self.mods.clone()
    }

    pub fn mod_entries(&self) -> Vec<ModEntryItem> {
        self.mod_entries.clone()
    }
}

#[derive(Debug, Clone)]
pub struct GameItem {
    handle: Game,
    pub name: String,
    pub active: bool,
    pub profiles: Vec<ProfileItem>,
}

impl PartialEq for GameItem {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl GameItem {
    pub fn handle(&self) -> Game {
        self.handle.clone()
    }

    async fn load(game: Game) -> anyhow::Result<Self> {
        Ok(Self {
            name: game.name().await?,
            active: game.is_active().await?,
            profiles: ProfileItem::load_all(&game).await?,
            handle: game,
        })
    }

    async fn load_active(repo: &Repository) -> anyhow::Result<Option<GameItem>> {
        let active_game = repo.active_game().await?;

        if let Some(game) = active_game {
            Ok(Some(GameItem::load(game).await?))
        } else {
            Ok(None)
        }
    }

    async fn load_all(repo: &Repository) -> anyhow::Result<Vec<Self>> {
        let games = repo.games().await?;

        try_join_all(games.into_iter().map(GameItem::load)).await
    }
}

#[derive(Debug, Clone, Display)]
#[display("{name}")]
pub struct ProfileItem {
    handle: Profile,
    pub name: String,
}

impl ProfileItem {
    pub fn handle(&self) -> Profile {
        self.handle.clone()
    }

    async fn load(profile: Profile) -> anyhow::Result<Self> {
        Ok(Self {
            name: profile.name().await?,
            handle: profile,
        })
    }

    async fn load_active(game: &Game) -> anyhow::Result<Option<ProfileItem>> {
        let active_profile = game.active_profile().await?;

        if let Some(profile) = active_profile {
            Ok(Some(ProfileItem::load(profile).await?))
        } else {
            Ok(None)
        }
    }

    async fn load_all(game: &Game) -> anyhow::Result<Vec<Self>> {
        let profiles = game.profiles().await?;

        try_join_all(profiles.into_iter().map(ProfileItem::load)).await
    }
}

#[derive(Debug, Clone)]
pub struct ModItem {
    handle: Mod,
    pub name: String,
}

impl ModItem {
    async fn load(mod_: Mod) -> anyhow::Result<Self> {
        Ok(Self {
            name: mod_.name().await?,
            handle: mod_,
        })
    }

    async fn load_all(game: &Game) -> anyhow::Result<Vec<Self>> {
        let mods = game.mods().await?;

        try_join_all(mods.into_iter().map(ModItem::load)).await
    }
}

#[derive(Debug, Clone)]
pub struct ModEntryItem {
    handle: ModEntry,
    pub name: String,
    pub enabled: bool,
}

impl ModEntryItem {
    pub fn handle(&self) -> ModEntry {
        self.handle.clone()
    }

    async fn load(mod_entry: ModEntry) -> anyhow::Result<Self> {
        Ok(Self {
            name: mod_entry.name().await?,
            enabled: mod_entry.enabled().await?,
            handle: mod_entry,
        })
    }

    async fn load_all(profile: &Profile) -> anyhow::Result<Vec<Self>> {
        let mod_entries = profile.mod_entries().await?;

        try_join_all(mod_entries.into_iter().map(ModEntryItem::load)).await
    }
}
