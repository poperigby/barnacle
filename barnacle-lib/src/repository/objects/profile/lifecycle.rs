use super::*;

impl Profile {
    /// Activate this profile.
    pub async fn activate(&self) -> Result<(), ActivateError> {
        let parent_game = self.parent().await.map_err(ActivateError::Parent)?;

        parent_game
            .set_active_profile_id(Some(self.id))
            .await
            .map_err(ActivateError::SetActiveProfile)?;

        Ok(())
    }

    pub async fn is_active(&self) -> Result<bool, IsActiveError> {
        let parent_game = self.parent().await.map_err(IsActiveError::Parent)?;

        let active_profile = Self::active(&self.db, &self.cfg, &parent_game)
            .await
            .map_err(IsActiveError::Active)?;

        Ok(active_profile.as_ref() == Some(self))
    }

    pub(crate) async fn active(
        db: &Db,
        cfg: &Cfg,
        game: &Game,
    ) -> Result<Option<Profile>, ActiveError> {
        Ok(Self::resolve_active_id(db, game)
            .await
            .map_err(ActiveError::Resolve)?
            .map(|id| Profile::from_id(id, db, cfg)))
    }

    // Returns the active profile ID, selecting a fallback if none is set.
    async fn resolve_active_id(db: &Db, game: &Game) -> Result<Option<i32>, ResolveActiveIdError> {
        if let Some(id) = game
            .active_profile_id()
            .await
            .map_err(ResolveActiveIdError::ActiveProfileId)?
        {
            // We already have an active profile
            return Ok(Some(id));
        }

        let conn = db.conn();

        // Make the oldest profile the fallback
        let fallback_id = Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_asc()
            .one(conn)
            .await
            .map_err(ResolveActiveIdError::FindFallbackProfile)?
            .map(|game| game.id);

        game.set_active_profile_id(fallback_id)
            .await
            .map_err(ResolveActiveIdError::SetActiveProfile)?;

        Ok(fallback_id)
    }

    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the profile is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        fs::remove_dir_all(&dir).map_err(|source| RemoveError::RemoveDir { path: dir, source })?;

        info!("Removed profile: {name}");

        Ok(())
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self, AddError> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id()),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    AddError::DuplicateName {
                        name: name.to_string(),
                    }
                } else {
                    AddError::Insert(source)
                }
            })?
            .last_insert_id;

        let profile = Profile::from_id(id, db, cfg);
        let dir = profile.dir().await.map_err(AddError::Dir)?;
        fs::create_dir_all(&dir).map_err(|source| AddError::CreateDir { path: dir, source })?;

        info!("Added profile: {name}");

        Ok(profile)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Profile::from_id(model.id, db, cfg))
            .collect())
    }

    /// Search for a profile under the given game by name.
    pub(crate) async fn search(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
    ) -> Result<Option<Profile>, SearchError> {
        Ok(
            Entity::find_by_profile_name_per_game((name.to_string(), game.id()))
                .one(db.conn())
                .await
                .map_err(|source| SearchError {
                    name: name.to_string(),
                    source,
                })?
                .map(|model| Profile::from_id(model.id, &db, &cfg)),
        )
    }
}
