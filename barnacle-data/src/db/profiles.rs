use agdb::QueryBuilder;

use crate::{
    db::{Database, GameId, ProfileId, Result},
    schema::v1::profiles::Profile,
};

impl Database {
    /// Insert a new Profile, linked to the given Game node
    pub fn insert_profile(&mut self, profile: &Profile, game_id: GameId) -> Result<ProfileId> {
        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(profile).query())?
                .elements[0]
                .id;

            // Link Profile to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id.0)
                    .to(profile_id)
                    .query(),
            )?;

            Ok(ProfileId(profile_id))
        })
    }

    pub fn current_profile(&self) -> Result<Profile> {
        let profile: Profile = self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Profile>()
                    .search()
                    .from("current_profile")
                    .query(),
            )?
            .try_into()?;

        Ok(profile)
    }

    pub fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // Delete existing current_profile, if it exists
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .from("current_profile")
                    .where_()
                    .edge()
                    .query(),
            )?;
            // Insert a new edge from current_profile to new profile_id
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from("current_profile")
                    .to(profile_id.0)
                    .query(),
            )?;

            Ok(())
        })
    }
}
