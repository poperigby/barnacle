use std::fs::create_dir_all;

use native_db::Database;

use crate::{data_dir, domain::v1::profiles::Profile};

pub struct ProfileRepo<'a> {
    db: Database<'a>,
}

impl<'a> ProfileRepo<'a> {
    pub fn new(db: Database<'a>) -> Self {
        Self { db }
    }

    pub fn add_profile(&self, name: &str) {
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
