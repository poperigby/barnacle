use std::path::PathBuf;

use barnacle_lib::{
    Repository,
    repository::{
        Game, Mod, ModEntry, Profile,
        item_entries::{ItemEntry, ItemEntryGroup},
    },
};
use derive_more::Display;
use futures::future::{join_all, try_join_all};

pub use mutation::Mutation;

mod mutation;

/// Runtime data used to render the loaded GUI.
///
/// Contains the in-memory projection of [`Repository`] state. Component `view()` functions can read from
/// this cheaply. [`Repository`] data is updated only from successful mutations.
#[derive(Debug, Clone)]
pub struct Model {
    active_game: Option<GameRecord>,
    games: Vec<GameRecord>,

    active_profile: Option<ProfileRecord>,

    mods: Vec<ModRecord>,
    mod_entries: Vec<ModEntryRecord>,

    item_entry_groups: Vec<ItemEntryGroupRecord>,
}

impl Model {
    pub async fn load(repo: &Repository) -> anyhow::Result<Self> {
        // We barely have anything to load if there's no active game (which means there aren't any games).
        let active_game_object = match repo.active_game().await? {
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

        let active_game = GameRecord::load_active(repo).await?;

        let games = GameRecord::load_all(repo).await?;

        let active_profile_object = match active_game_object.active_profile().await? {
            Some(profile) => profile,
            None => {
                return Ok(Self::new(
                    active_game,
                    games,
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ));
            }
        };

        let active_profile = ProfileRecord::load(active_profile_object.clone()).await?;

        let mods = ModRecord::load_all(&active_game_object).await?;
        let mod_entries = ModEntryRecord::load_all(&active_profile_object).await?;

        let item_entry_groups = ItemEntryGroupRecord::load_all(&active_profile_object).await;

        Ok(Self::new(
            active_game,
            games,
            Some(active_profile),
            mods,
            mod_entries,
            item_entry_groups,
        ))
    }

    fn new(
        active_game: Option<GameRecord>,
        games: Vec<GameRecord>,
        active_profile: Option<ProfileRecord>,
        mods: Vec<ModRecord>,
        mod_entries: Vec<ModEntryRecord>,
        item_entry_groups: Vec<ItemEntryGroupRecord>,
    ) -> Self {
        Self {
            active_game,
            games,
            active_profile,
            mods,
            mod_entries,
            item_entry_groups,
        }
    }

    pub fn active_game(&self) -> Option<GameRecord> {
        self.active_game.clone()
    }

    pub fn games(&self) -> Vec<GameRecord> {
        self.games.clone()
    }

    pub fn game(&self, game: &Game) -> Option<GameRecord> {
        self.games()
            .iter()
            .find(|record| record.object() == *game)
            .cloned()
    }

    pub fn active_profile(&self) -> Option<ProfileRecord> {
        self.active_profile.clone()
    }

    pub fn profile(&self, profile: &Profile) -> Option<ProfileRecord> {
        self.active_game()?
            .profiles
            .iter()
            .find(|record| record.object() == *profile)
            .cloned()
    }

    pub fn mods(&self) -> Vec<ModRecord> {
        self.mods.clone()
    }

    pub fn mod_entries(&self) -> Vec<ModEntryRecord> {
        self.mod_entries.clone()
    }

    pub fn item_entry_groups(&self) -> Vec<ItemEntryGroupRecord> {
        self.item_entry_groups.clone()
    }
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    object: Game,
    pub name: String,
    pub active: bool,
    pub profiles: Vec<ProfileRecord>,
}

impl PartialEq for GameRecord {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl GameRecord {
    pub fn object(&self) -> Game {
        self.object.clone()
    }

    async fn load(game: Game) -> anyhow::Result<Self> {
        Ok(Self {
            name: game.name().await?,
            active: game.is_active().await?,
            profiles: ProfileRecord::load_all(&game).await?,
            object: game,
        })
    }

    async fn load_active(repo: &Repository) -> anyhow::Result<Option<GameRecord>> {
        let active_game = repo.active_game().await?;

        if let Some(game) = active_game {
            Ok(Some(GameRecord::load(game).await?))
        } else {
            Ok(None)
        }
    }

    async fn load_all(repo: &Repository) -> anyhow::Result<Vec<Self>> {
        let games = repo.games().await?;

        try_join_all(games.into_iter().map(GameRecord::load)).await
    }
}

#[derive(Debug, Clone, Display)]
#[display("{name}")]
pub struct ProfileRecord {
    object: Profile,
    pub name: String,
}

impl ProfileRecord {
    pub fn object(&self) -> Profile {
        self.object.clone()
    }

    async fn load(profile: Profile) -> anyhow::Result<Self> {
        Ok(Self {
            name: profile.name().await?,
            object: profile,
        })
    }

    async fn load_active(game: &Game) -> anyhow::Result<Option<ProfileRecord>> {
        let active_profile = game.active_profile().await?;

        if let Some(profile) = active_profile {
            Ok(Some(ProfileRecord::load(profile).await?))
        } else {
            Ok(None)
        }
    }

    async fn load_all(game: &Game) -> anyhow::Result<Vec<Self>> {
        let profiles = game.profiles().await?;

        try_join_all(profiles.into_iter().map(ProfileRecord::load)).await
    }
}

#[derive(Debug, Clone)]
pub struct ModRecord {
    object: Mod,
    pub name: String,
}

impl ModRecord {
    async fn load(mod_: Mod) -> anyhow::Result<Self> {
        Ok(Self {
            name: mod_.name().await?,
            object: mod_,
        })
    }

    async fn load_all(game: &Game) -> anyhow::Result<Vec<Self>> {
        let mods = game.mods().await?;

        try_join_all(mods.into_iter().map(ModRecord::load)).await
    }
}

#[derive(Debug, Clone)]
pub struct ModEntryRecord {
    object: ModEntry,
    pub name: String,
    pub enabled: bool,
}

impl ModEntryRecord {
    pub fn object(&self) -> ModEntry {
        self.object.clone()
    }

    async fn load(mod_entry: ModEntry) -> anyhow::Result<Self> {
        Ok(Self {
            name: mod_entry.name().await?,
            enabled: mod_entry.enabled().await?,
            object: mod_entry,
        })
    }

    async fn load_all(profile: &Profile) -> anyhow::Result<Vec<Self>> {
        let mod_entries = profile.mod_entries().await?;

        try_join_all(mod_entries.into_iter().map(ModEntryRecord::load)).await
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntryRecord {
    object: ItemEntry,
    pub relative_path: PathBuf,
    pub enabled: bool,
}

impl ItemEntryRecord {
    async fn load(item_entry: &ItemEntry) -> Self {
        Self {
            object: item_entry.clone(),
            relative_path: item_entry.relative_path().await,
            enabled: item_entry.enabled().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntryGroupRecord {
    object: ItemEntryGroup,
    pub name: String,
    pub ordered: bool,
    pub entries: Vec<ItemEntryRecord>,
}

impl ItemEntryGroupRecord {
    async fn load_all(profile: &Profile) -> Vec<ItemEntryGroupRecord> {
        let item_entry_groups = profile.item_entry_groups().await;

        join_all(item_entry_groups.iter().map(ItemEntryGroupRecord::load)).await
    }

    async fn load(item_entry_group: &ItemEntryGroup) -> Self {
        let entries = join_all(item_entry_group.entries.iter().map(ItemEntryRecord::load)).await;

        Self {
            object: item_entry_group.clone(),
            name: item_entry_group.name.clone(),
            ordered: item_entry_group.ordered,
            entries,
        }
    }
}
