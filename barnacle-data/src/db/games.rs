use agdb::{CountComparison, QueryBuilder};

use crate::{
    db::{Database, DatabaseError, GameId, Result, UniqueConstraint},
    schema::v1::games::Game,
};

impl Database {
    /// Insert a new Game into the database. The Game must have a unique name.
    pub fn insert_game(&mut self, game: &Game) -> Result<GameId> {
        if self.games()?.iter().any(|g| g.name() == game.name()) {
            return Err(DatabaseError::UniqueViolation(UniqueConstraint::GameName));
        }

        self.0.transaction_mut(|t| {
            let game_id = t
                .exec_mut(QueryBuilder::insert().element(game).query())?
                .elements[0]
                .id;

            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from("games")
                    .to(game_id)
                    .query(),
            )?;

            Ok(GameId(game_id))
        })
    }

    pub fn games(&self) -> Result<Vec<Game>> {
        Ok(self
            .0
            .exec(
                QueryBuilder::select()
                    .elements::<Game>()
                    .search()
                    .from("games")
                    .where_()
                    // Make sure we're not grabbing edges
                    .node()
                    .and()
                    // Don't count the first node, which is the alias
                    .distance(CountComparison::GreaterThan(1))
                    .query(),
            )?
            .try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::v1::games::DeployKind;

    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    fn setup_db() -> Database {
        let tmp_dir = tempdir().unwrap();
        let mut db = Database::new(&tmp_dir.path().join("test.db")).unwrap();
        db.init().unwrap();

        db
    }

    #[test]
    fn test_insert_game() {
        let mut db = setup_db();

        let game = Game::new("Morrowind", DeployKind::OpenMW);

        db.insert_game(&game).unwrap();

        let games = db.games().unwrap();
        let inserted_game = games.first().unwrap();

        assert_eq!(inserted_game.name(), "Morrowind");
        assert_eq!(inserted_game.deploy_kind(), DeployKind::OpenMW);
    }

    #[test]
    fn test_games() {
        let mut db = setup_db();

        let game1 = Game::new("Morrowind", DeployKind::OpenMW);
        let game2 = Game::new("Skyrim", DeployKind::Gamebryo);

        db.insert_game(&game1).unwrap();
        db.insert_game(&game2).unwrap();

        let games = db.games().unwrap();

        let names: Vec<_> = games.iter().map(|g| g.name()).collect();
        assert!(names.contains(&"Morrowind"));
        assert!(names.contains(&"Skyrim"));

        let deploy_kinds: Vec<_> = games.iter().map(|g| g.deploy_kind()).collect();
        assert!(deploy_kinds.contains(&DeployKind::OpenMW));
        assert!(deploy_kinds.contains(&DeployKind::Gamebryo));
    }

    #[test]
    fn test_games_empty() {
        let db = setup_db();
        let games = db.games().unwrap();
        assert!(games.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_insert_duplicate_game() {
        let mut db = setup_db();
        let game = Game::new("Morrowind", DeployKind::OpenMW);

        db.insert_game(&game).unwrap();
        db.insert_game(&game).unwrap();
    }
}
