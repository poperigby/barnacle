use std::path::Path;

use agdb::{DbAny, QueryBuilder};

use crate::{
    Error, Result,
    models::{CURRENT_MODEL_VERSION, ModelVersion},
};

pub mod games;
pub mod mods;
pub mod profiles;
pub mod tools;

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(DbAny);

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or(Error::PathInvalidUnicode)?;
        Self::init(DbAny::new_file(path_str)?)
    }

    /// Create a memory backed database for use in tests
    #[allow(dead_code)]
    fn new_memory() -> Result<Self> {
        Self::init(DbAny::new_memory("data.db")?)
    }

    fn init(mut db: DbAny) -> Result<Self> {
        // Insert aliases if they don't exist
        if db.exec(QueryBuilder::select().aliases().query())?.result == 0 {
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
            )?;
        }

        // Fetch the current model version (if any)
        let result = db.exec(
            QueryBuilder::select()
                .elements::<ModelVersion>()
                .search()
                .from("model_version")
                .where_()
                .neighbor()
                .query(),
        )?;

        let model_version: Option<ModelVersion> = result.try_into().into_iter().next();

        if let Some(mv) = model_version {
            if mv.version() < CURRENT_MODEL_VERSION {
                // TODO: perform migrations
                dbg!(mv);
            }
        } else {
            // Insert default ModelVersion if missing
            db.transaction_mut(|t| -> Result<()> {
                let model_version_id = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .element(ModelVersion::default())
                            .query(),
                    )?
                    .elements
                    .first()
                    .ok_or(Error::EmptyInsertResult)?
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from("model_version")
                        .to(model_version_id)
                        .query(),
                )?;

                Ok(())
            })?;
        }

        Ok(Database(db))
    }
}
