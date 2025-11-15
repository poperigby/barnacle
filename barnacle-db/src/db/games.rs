use agdb::QueryBuilder;

use crate::{Error, GameId, Result, UniqueConstraint, db::Database, models::Game};

impl Database {
    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub async fn insert_game(&mut self, game: &Game) -> Result<GameId> {
        if self.games().await?.iter().any(|g| g.name() == game.name()) {
            return Err(Error::UniqueViolation(UniqueConstraint::GameName));
        }

        self.0.write().await.transaction_mut(|t| {
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

    /// Retrieve a [`Game`] by ID
    pub async fn game(&self, game_id: GameId) -> Result<Game> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().ids(game_id.0).query())?
            .try_into()?)
    }

    /// Retrieve the list of all [`Game`]s
    pub async fn games(&self) -> Result<Vec<Game>> {
        Ok(self
            .0
            .read()
            .await
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
                    .neighbor()
                    .query(),
            )?
            .try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::DeployKind;

    use super::*;
    use macro_rules_attribute::apply;
    use pretty_assertions::assert_eq;
    use smol_macros::test;

    #[apply(test!)]
    async fn test_insert_game() {
        let mut db = Database::new_memory().unwrap();

        let game = Game::new("Morrowind", DeployKind::OpenMW);

        db.insert_game(&game).await.unwrap();

        let games = db.games().await.unwrap();
        let inserted_game = games.first().unwrap();

        assert_eq!(inserted_game.name(), "Morrowind");
        assert_eq!(inserted_game.deploy_kind(), DeployKind::OpenMW);
    }

    #[apply(test!)]
    async fn test_games() {
        let mut db = Database::new_memory().unwrap();

        let game1 = Game::new("Morrowind", DeployKind::OpenMW);
        let game2 = Game::new("Skyrim", DeployKind::Gamebryo);

        db.insert_game(&game1).await.unwrap();
        db.insert_game(&game2).await.unwrap();

        let games = db.games().await.unwrap();

        let names: Vec<_> = games.iter().map(|g| g.name()).collect();
        assert!(names.contains(&"Morrowind"));
        assert!(names.contains(&"Skyrim"));

        let deploy_kinds: Vec<_> = games.iter().map(|g| g.deploy_kind()).collect();
        assert!(deploy_kinds.contains(&DeployKind::OpenMW));
        assert!(deploy_kinds.contains(&DeployKind::Gamebryo));
    }

    #[apply(test!)]
    async fn test_games_empty() {
        let db = Database::new_memory().unwrap();
        let games = db.games().await.unwrap();
        assert!(games.is_empty());
    }

    #[apply(test!)]
    async fn test_insert_duplicate_game() {
        let mut db = Database::new_memory().unwrap();
        let game = Game::new("Morrowind", DeployKind::OpenMW);

        db.insert_game(&game).await.unwrap();
        assert_eq!(
            db.insert_game(&game).await.unwrap_err(),
            Error::UniqueViolation(UniqueConstraint::GameName)
        );
    }
}
