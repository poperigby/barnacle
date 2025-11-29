use std::{path::Path, sync::Arc};

use agdb::{DbAny, DbError, DbId, DbValue, QueryBuilder};
use derive_more::Deref;
use tokio::sync::RwLock;

use crate::repository::models::{CURRENT_MODEL_VERSION, ModelVersion};

mod games;
mod mods;
mod profiles;
mod tools;

#[derive(Debug, Clone, Deref)]
pub struct DbHandle {
    #[deref]
    db: Arc<RwLock<DbAny>>,
}

impl DbHandle {
    pub fn new(path: &Path) -> Self {
        let path_str = path.to_str().unwrap();
        Self::init(DbAny::new_file(path_str).unwrap())
    }

    /// Create a memory backed database for use in tests
    #[allow(dead_code)]
    fn new_memory() -> Self {
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

pub(crate) fn get_field<T: TryFrom<DbValue>>(
    db: &DbAny,
    field: &str,
    db_id: DbId,
) -> Result<T, DbError> {
    db.exec(QueryBuilder::select().values(field).ids(db_id).query())?
        .elements
        .pop()
        .expect("successful result values cannot be empty")
        .values
        .pop()
        .expect("successful result values cannot be empty")
        .value
        .try_into()
        .map_err(|_e| DbError::from(format!("Failed to convert '{field}'"))) //your error type
}
