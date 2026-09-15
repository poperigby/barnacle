use super::*;

impl Mod {
    pub(crate) fn new_mod(db: Db, cfg: Cfg, game: &Game, name: &str) -> NewMod {
        NewMod::new(&db, &cfg, game, name)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Mod::from_id(model.id, db, cfg))
            .collect())
    }

    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the mod is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        fs::remove_dir_all(&dir).map_err(|source| RemoveError::RemoveDir { path: dir, source })?;

        info!("Removed mod: {name}");

        Ok(())
    }
}
