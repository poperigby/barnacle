use barnacle_data::v1::{
    games::{Game, GameId},
    mods::{Mod, ModId},
    profiles::{Profile, ProfileId},
};

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

    pub fn insert_profile(&self, new_profile: Profile) {
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

        rw.remove(found_mod).unwrap();
        rw.commit().unwrap();
    }
}
