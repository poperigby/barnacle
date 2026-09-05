use barnacle_lib::{
    Repository,
    repository::{Game, Profile},
};
use derive_more::{Deref, Display};

#[derive(Debug, Clone)]
pub struct AppData {
    pub active_game: Option<Game>,
    pub active_profile: Option<ProfileOption>,

    pub profile_options: Vec<ProfileOption>,
}

impl AppData {
    pub async fn load(repo: &Repository) -> Self {
        let mut active_game = None;
        let mut active_profile = None;
        let mut profile_options = Vec::new();

        if let Some(game) = repo.active_game().await.unwrap() {
            active_game = Some(game.clone());
            for profile in game.profiles().await.unwrap() {
                if profile.is_active().await.unwrap() {
                    active_profile = Some(ProfileOption::new(profile.clone()).await)
                };

                let profile_option = ProfileOption {
                    handle: profile.clone(),
                    name: profile.name().await.unwrap(),
                };

                profile_options.push(profile_option);
            }
        };

        AppData {
            active_game,
            active_profile,
            profile_options,
        }
    }
}

#[derive(Clone, Debug, Display, Deref)]
#[display("{}", name)]
pub struct ProfileOption {
    #[deref]
    pub handle: Profile,
    pub name: String,
}

impl ProfileOption {
    pub async fn new(profile: Profile) -> Self {
        let name = profile.name().await.unwrap();
        Self {
            handle: profile,
            name,
        }
    }
}
