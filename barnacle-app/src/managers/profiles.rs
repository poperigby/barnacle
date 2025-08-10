use std::fs::create_dir_all;

use barnacle_data::v1::profiles::Profile;
use native_db::Database;

use crate::data_dir;

pub struct ProfilesManager<'a> {
    db: &'a Database<'a>,
}

impl<'a> ProfilesManager<'a> {
    pub fn new(db: &'a Database<'a>) -> Self {
        Self { db }
    }

    pub fn add(&self, name: &str) {
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
}
