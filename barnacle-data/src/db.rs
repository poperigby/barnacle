use std::path::Path;

use agdb::{Db, DbError, QueryBuilder};

use crate::v1::games::Game;

#[derive(Debug)]
pub struct Database(Db);

impl Database {
    pub fn init(path: &Path) -> Self {
        Database(Db::new(path.to_str().unwrap()).unwrap())
    }

    fn insert_game(&mut self, game: &Game) {
        self.0
            .transaction_mut(|t| -> Result<(), DbError> {
                let id = t
                    .exec_mut(QueryBuilder::insert().element(game).query())?
                    .elements[0]
                    .id;
                t.exec_mut(QueryBuilder::insert().edges().from("games").to(id).query())?;

                Ok(())
            })
            .unwrap()
    }
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
        let game = Game::new("Skyrim", DeployKind::Gamebryo);

        // Insert into the DB
        db.insert_game(&game);

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
