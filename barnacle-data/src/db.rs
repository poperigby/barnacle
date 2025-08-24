use std::path::Path;

use agdb::{Db, DbError, DbId, QueryBuilder};
use thiserror::Error;

use crate::v1::{
    games::Game,
    mods::{Mod, ModEntry},
    profiles::Profile,
};

type Result<T> = std::result::Result<T, DatabaseError>;

/// ID representing a Game in the database
#[derive(Debug)]
pub struct GameId(DbId);

/// ID representing a Profile in the database
#[derive(Debug)]
pub struct ProfileId(DbId);

/// ID representing a Mod in the database
#[derive(Debug)]
pub struct ModId(DbId);

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(Db);

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
}

impl Database {
    pub fn new(path: &Path) -> Self {
        Database(Db::new(path.to_str().unwrap()).unwrap())
    }

    /// Initialize the root nodes
    pub fn init(&mut self) -> Result<()> {
        self.0.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["games", "current_profile"])
                .query(),
        )?;

        Ok(())
    }

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

    /// Insert a new Profile, linked to the given Game node
    pub fn insert_profile(&mut self, profile: &Profile, game_name: &str) -> Result<ProfileId> {
        self.0.transaction_mut(|t| {
            let profile_id = t
                .exec_mut(QueryBuilder::insert().element(profile).query())?
                .elements[0]
                .id;

            // Look up game by name
            let game_id = t
                .exec(
                    QueryBuilder::select()
                        .search()
                        .from("games")
                        .where_()
                        .key("name")
                        .value(game_name)
                        .query(),
                )?
                .elements[0]
                .id;

            // Link Profile to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id)
                    .to(profile_id)
                    .query(),
            )?;
            Ok(ProfileId(profile_id))
        })
    }

    pub fn set_current_profile(&mut self, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // Check if there is already a current_profile
            t.exec(
                QueryBuilder::select()
                    .edge_count()
                    .search()
                    .from("current_profile")
                    .query(),
            )?;
            // If there is, delete the edge to it
            // Insert a new edge from current_profile to new profile_id

            Ok(())
        })
    }

    /// Insert a new Mod, linked to the given Game node
    pub fn insert_mod(&mut self, new_mod: &Mod, game_id: GameId) -> Result<ModId> {
        self.0.transaction_mut(|t| {
            let mod_id = t
                .exec_mut(QueryBuilder::insert().element(new_mod).query())?
                .elements[0]
                .id;

            // Link Mod to the specified Game node
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(game_id.0)
                    .to(mod_id)
                    .query(),
            )?;
            Ok(ModId(mod_id))
        })
    }

    pub fn link_mod_to_profile(&mut self, mod_id: ModId, profile_id: ProfileId) -> Result<()> {
        self.0.transaction_mut(|t| {
            // We don't have to worry about adding mods from the wrong game, because you can only query
            // a Mod from a Game node.
            let mod_entry = ModEntry::default();
            let mod_entry_id = t
                .exec_mut(QueryBuilder::insert().element(&mod_entry).query())?
                .elements[0]
                .id;

            // Insert ModEntry in between Profile and Mod
            // Profile -> ModEntry -> Mod
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(profile_id.0)
                    .to(mod_entry_id)
                    .query(),
            )?;
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(mod_entry_id)
                    .to(mod_id.0)
                    .query(),
            )?;

            Ok(())
        })
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
        let mut db = Database::new(&tmp_dir.path().join("test.db"));

        // Insert root "games" node (required before linking edges)
        db.init().unwrap();
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
