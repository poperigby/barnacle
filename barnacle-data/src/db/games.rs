use agdb::{CountComparison, QueryBuilder};

use crate::{
    db::{Database, GameId, Result},
    schema::v1::games::Game,
};

impl Database {
    pub fn insert_game(&mut self, game: &Game) -> Result<GameId> {
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
                    .node()
                    .and()
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
        assert!(matches!(inserted_game.deploy_kind(), DeployKind::OpenMW));
    }
}
