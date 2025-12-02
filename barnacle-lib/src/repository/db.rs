use std::sync::Arc;

use agdb::{DbAny, QueryBuilder};
use derive_more::Deref;
use parking_lot::RwLock;

use crate::{
    fs::data_dir,
    repository::models::{CURRENT_MODEL_VERSION, ModelVersion},
};

#[derive(Debug, Clone, Deref)]
pub(crate) struct DbHandle {
    #[deref]
    db: Arc<RwLock<DbAny>>,
}

impl DbHandle {
    pub fn new() -> Self {
        let path = data_dir().join("data.db");
        let path_str = path.to_str().unwrap();
        Self::init(DbAny::new_file(path_str).unwrap())
    }

    /// Create a memory backed database for use in tests
    #[allow(dead_code)]
    pub(crate) fn new_memory() -> Self {
        Self::init(DbAny::new_memory("data.db").unwrap())
    }

    fn init(mut db: DbAny) -> Self {
        // Insert aliases if they don't exist
        if db
            .exec(QueryBuilder::select().aliases().query())
            .unwrap()
            .result
            == 0
        {
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases([
                        "games",
                        "profiles",
                        "mods",
                        "tools",
                        // State
                        "current_profile",
                        "model_version",
                    ])
                    .query(),
            )
            .unwrap();
        }

        // Fetch the current model version (if any)
        let result = db
            .exec(
                QueryBuilder::select()
                    .elements::<ModelVersion>()
                    .search()
                    .from("model_version")
                    .where_()
                    .neighbor()
                    .query(),
            )
            .unwrap();

        let model_version: Option<ModelVersion> = result.try_into().into_iter().next();

        if let Some(mv) = model_version {
            if mv.version() < CURRENT_MODEL_VERSION {
                // TODO: perform migrations
                dbg!(mv);
            }
        } else {
            // Insert default ModelVersion if missing
            db.transaction_mut(|t| -> Result<(), agdb::DbError> {
                let model_version_id = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .element(ModelVersion::default())
                            .query(),
                    )?
                    .elements
                    .first()
                    .unwrap()
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from("model_version")
                        .to(model_version_id)
                        .query(),
                )?;

                Ok(())
            })
            .unwrap();
        }

        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}

// pub(crate) fn set_field<T>(db: &DbHandle, id: DbId, field: &str, value: &T) -> Result<(), DbError>
// where
//     T: Into<DbValue>,
// {
//     db.write().exec_mut(
//         QueryBuilder::insert()
//             .values([[(field, value).into()]])
//             .ids(id)
//             .query(),
//     )?;
//
//     Ok(())
// }
