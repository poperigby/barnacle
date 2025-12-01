use std::path::PathBuf;

use agdb::{DbId, QueryBuilder, QueryId};

use crate::repository::{
    CoreConfigHandle,
    db::DbHandle,
    entities::{Error, Result, get_field, mod_::Mod, profile::Profile, set_field},
    models::{DeployKind, ProfileModel},
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Game {
    id: DbId,
    db: DbHandle,
    cfg: CoreConfigHandle,
}

impl Game {
    pub(crate) fn from_id(id: DbId, db: DbHandle, cfg: CoreConfigHandle) -> Self {
        Self { id, db, cfg }
    }

    pub fn name(&self) -> Result<String> {
        get_field(&self.db, self.id, "name")
    }

    pub fn set_name(&mut self, new_name: &str) -> Result<()> {
        set_field(&mut self.db, self.id, "name", new_name)
    }

    pub fn targets(&self) -> Result<Vec<PathBuf>> {
        get_field(&self.db, self.id, "targets")
    }

    pub fn deploy_kind(&self) -> Result<DeployKind> {
        get_field(&self.db, self.id, "deploy_kind")
    }

    pub fn set_deploy_kind(&mut self, new_deploy_kind: DeployKind) -> Result<()> {
        set_field(&mut self.db, self.id, "deploy_kind", new_deploy_kind)
    }

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self.cfg.read().game_dir(&self.name()?))
    }

    pub fn add_profile(&mut self, name: &str) -> Result<Profile> {
        let new_profile = ProfileModel::new(name);

        if self
            .profiles()?
            .iter()
            .any(|p: &Profile| p.name().unwrap() == new_profile.name)
        {
            // return Err(Error::UniqueViolation(UniqueConstraint::ProfileName));
            panic!("Unique violation")
        }

        Ok(self.db.write().transaction_mut(|t| -> Result<Profile> {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(new_profile).query())?
                .elements
                .first()
                .ok_or(Error::EmptyElements)?
                .id;

            // Link Profile to the specified Game node and root "profiles" node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(self.id)])
                    .to(profile_id)
                    .query(),
            )?;

            Ok(Profile::from_id(profile_id, self.db.clone()))
        })?)
    }

    pub fn profiles(&self) -> Result<Vec<Profile>> {
        Ok(self
            .db
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
            )?
            .elements
            .iter()
            .map(|e| Profile::from_id(e.id, self.db.clone()))
            .collect())
    }

    pub fn add_mod(&mut self, name: &str) -> Result<Mod> {
        todo!()
        // // TODO: Only attempt to open the archive if the input_path is an archive
        // if let Some(path) = input_path {
        //     let archive = File::open(path)?;
        //     uncompress_archive(archive, &dir, Ownership::Preserve)?;
        //     change_dir_permissions(&dir, Permissions::ReadOnly);
        // }
    }
}
