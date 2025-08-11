// TODO: Move business logic out of here

use std::fs::{create_dir_all, remove_dir_all};

use barnacle_data::v1::{
    games::{Game, GameId},
    mods::{Mod, ModId},
    profiles::{Profile, ProfileId},
};

use crate::{Permissions, change_dir_permissions, data_dir};

/// Client for performing operations on the database
pub struct Database<'a> {
    db: native_db::Database<'a>,
}

impl<'a> Database<'a> {
    pub fn new(db: native_db::Database<'a>) -> Self {
        Self { db }
    }

    pub fn insert_game(&self, new_game: Game) {
        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_game).unwrap();
        rw.commit().unwrap();
    }

    pub fn remove_game(&self, id: GameId) {
        let rw = self.db.rw_transaction().unwrap();
        let found_game: Game = rw.get().primary(id).unwrap().unwrap();

        rw.remove(found_game).unwrap();
        rw.commit().unwrap();
    }

    pub fn insert_profile(&self, name: &str) {
        let new_profile = Profile::new(name);

        create_dir_all(
            data_dir()
                .join("profiles")
                .join(new_profile.id().to_string()),
        )
        .unwrap();

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_profile).unwrap();
        rw.commit().unwrap();
    }

    pub fn remove_profile(&self, id: ProfileId) {
        let rw = self.db.rw_transaction().unwrap();
        let found_profile: Profile = rw.get().primary(id).unwrap().unwrap();

        rw.remove(found_profile).unwrap();
        rw.commit().unwrap();
    }

    pub fn insert_mod(&self, new_mod: Mod) {
        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_mod).unwrap();
        rw.commit().unwrap();
    }

    pub fn remove_mod(&self, id: ModId) {
        let rw = self.db.rw_transaction().unwrap();
        let found_mod: Mod = rw.get().primary(id).unwrap().unwrap();
        let dir = data_dir().join("mods").join(found_mod.id().to_string());

        change_dir_permissions(&dir, Permissions::ReadWrite);
        remove_dir_all(&dir).unwrap();

        rw.remove(found_mod).unwrap();
        rw.commit().unwrap();
    }
}
