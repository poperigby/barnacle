use std::path::PathBuf;

use agdb::{DbId, QueryBuilder, QueryId};

use crate::repository::{
    db::{DbHandle, get_field},
    entities::profiles::Profile,
    models::{DeployKind, ProfileModel},
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
pub struct Game {
    id: DbId,
    db: DbHandle,
}

impl Game {
    pub fn new(id: DbId, db: DbHandle) -> Self {
        Self { id, db }
    }

    pub fn name(&self) -> String {
        let db = self.db.read();

        get_field(&db, "name", self.id).unwrap()
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        let db = self.db.read();

        get_field(&db, "targets", self.id).unwrap()
    }

    pub fn deploy_kind(&self) -> DeployKind {
        let db = self.db.read();

        get_field(&db, "deploy_kind", self.id).unwrap()
    }

    pub fn add_profile(&mut self, name: &str) -> Profile {
        let new_profile = ProfileModel::new(name);

        if self
            .profiles()
            .iter()
            .any(|p: &Profile| p.name() == new_profile.name)
        {
            // return Err(Error::UniqueViolation(UniqueConstraint::ProfileName));
            panic!("Unique violation")
        }

        self.db
            .write()
            .transaction_mut(|t| -> Result<Profile, agdb::DbError> {
                let profile_id = t
                    .exec_mut(QueryBuilder::insert().element(new_profile).query())?
                    .elements
                    .first()
                    .unwrap()
                    .id;

                // Link Profile to the specified Game node and root "profiles" node
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from([QueryId::from("profiles"), QueryId::from(self.id)])
                        .to(profile_id)
                        .query(),
                )?;

                Ok(Profile::new(profile_id, self.db.clone()))
            })
            .unwrap()
    }

    pub fn profiles(&self) -> Vec<Profile> {
        self.db
            .read()
            .exec(
                QueryBuilder::select()
                    .elements::<ProfileModel>()
                    .search()
                    .from(self.id)
                    .where_()
                    .node()
                    .and()
                    .neighbor()
                    .query(),
            )
            .unwrap()
            .elements
            .iter()
            .map(|e| Profile::new(e.id, self.db.clone()))
            .collect()
    }
}
