use std::path::Path;

use agdb::{Db, DbError, DbId, QueryBuilder, QueryId};

use crate::v1::{games::Game, mods::Mod, profiles::Profile};

#[derive(Debug)]
pub struct Database(Db);

impl Database {
    pub fn init(path: &Path) -> Self {
        Database(Db::new(path.to_str().unwrap()).unwrap())
    }

    pub fn insert_game(&mut self, game: &Game) -> Result<DbId, DbError> {
        self.0.transaction_mut(|t| -> Result<DbId, DbError> {
            let id = t
                .exec_mut(QueryBuilder::insert().element(game).query())?
                .elements[0]
                .id;

            // Link Game to "games" root node
            t.exec_mut(QueryBuilder::insert().edges().from("games").to(id).query())?;

            Ok(id)
        })
    }

    /// Insert a new Profile, linked to the given Game node
    pub fn insert_profile(&mut self, profile: &Profile, game_id: DbId) -> Result<DbId, DbError> {
        self.0.transaction_mut(|t| -> Result<DbId, DbError> {
            let id = t
                .exec_mut(QueryBuilder::insert().element(profile).query())?
                .elements[0]
                .id;

            // Link Profile to "profiles" root node and to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("profiles"), QueryId::from(game_id)])
                    .to(id)
                    .query(),
            )?;
            Ok(id)
        })
    }

    /// Insert a new Mod, linked to the given Game node
    pub fn insert_mod(&mut self, new_mod: &Mod, game_id: DbId) -> Result<DbId, DbError> {
        self.0.transaction_mut(|t| -> Result<DbId, DbError> {
            let id = t
                .exec_mut(QueryBuilder::insert().element(new_mod).query())?
                .elements[0]
                .id;

            // Link Mod to "mods" root node and to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from("mods"), QueryId::from(game_id)])
                    .to(id)
                    .query(),
            )?;
            Ok(id)
        })
    }

    // pub fn link_mod_to_profile(&mut self, mod_id: DbId, profile_id: DbId) -> Result<DbId, DbError> {
    //     self.0.transaction_mut(|t| -> Result<DbId, DbError> {
    //         // Find ID of the game the profile is connected to
    //         // Search for
    //
    //
    //         let found_mod = t.exec(QueryBuilder::select().elements::<Mod>())
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use crate::v1::games::DeployKind;

    use super::*;
    use agdb::QueryBuilder;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_insert_game() {
        let tmp_dir = tempdir().unwrap();
        let mut db = Database::init(&tmp_dir.path().join("test.db"));

        // Insert root "games" node (required before linking edges)
        db.0.exec_mut(QueryBuilder::insert().nodes().aliases(["games"]).query())
            .expect("Failed to insert root 'game' node");

        // Create a Game instance
        let game = Game::new("Morrowind", DeployKind::OpenMW);

        db.insert_game(&game).unwrap();

        // Query all games linked under "games"
        let games: Vec<Game> =
            db.0.exec(
                QueryBuilder::select()
                    .elements::<Game>()
                    .search()
                    .from("games")
                    .query(),
            )
            .unwrap()
            .try_into()
            .unwrap();

        // Ensure one game exists and has the correct name and deploy type
        // let inserted_game: &Game = &games[0];
        // assert_eq!(inserted_game.name, "Skyrim");
        // assert!(matches!(inserted_game.deploy_type, DeployType::Gamebryo));
    }
}
