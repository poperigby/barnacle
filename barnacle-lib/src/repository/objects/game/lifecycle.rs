use super::*;

impl Game {
    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the game is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        // We want to roll back the database mutations if the directory removal fails
        self.db
            .conn()
            .transaction(|txn| {
                Box::pin(async move {
                    Entity::delete_by_id(self.id)
                        .exec(txn)
                        .await
                        .map_err(RemoveError::Delete)?;

                    fs::remove_dir_all(dir).map_err(|source| RemoveError::RemoveDir { source })?;

                    Ok::<(), RemoveError>(())
                })
            })
            .await
            .map_err(|source| map_transaction_error(source, RemoveError::Transaction))?;

        info!("Removed game: {name}");

        Ok(())
    }

    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub(crate) async fn add(
        db: &Db,
        cfg: &Cfg,
        name: &str,
        deploy_kind: DeployKind,
    ) -> Result<Self, AddError> {
        let name = name.to_string();

        let launch_program = match deploy_kind {
            DeployKind::OpenMW => "openmw",
            DeployKind::Skyrim => "skyrim",
            DeployKind::FalloutNV => "fallout_nv",
        };

        let model = ActiveModel {
            name: Set(name.clone()),
            deploy_kind: Set(deploy_kind),
            launch_program: Set(launch_program.to_string()),
            launch_args: Set("".to_string()),
            ..Default::default()
        };

        // We want to roll back the database mutations if the directory creation fails
        let id = db
            .conn()
            .transaction(|txn| {
                let name = name.clone();
                let cfg = cfg.clone();

                Box::pin(async move {
                    let id = Entity::insert(model)
                        .exec(txn)
                        .await
                        .map_err(|source| {
                            if is_unique_violation(&source) {
                                AddError::DuplicateName { name: name.clone() }
                            } else {
                                AddError::Insert(source)
                            }
                        })?
                        .last_insert_id;

                    let dir = Self::dir_from_id(&cfg, id);
                    fs::create_dir_all(&dir)
                        .map_err(|source| AddError::CreateDir { path: dir, source })?;

                    Ok(id)
                })
            })
            .await
            .map_err(|source| map_transaction_error(source, AddError::Transaction))?;

        info!("Created new game: {name}");

        Ok(Game::from_id(id, db, cfg))
    }

    pub(crate) async fn list(db: Db, cfg: Cfg) -> Result<Vec<Game>, ListError> {
        Ok(Entity::find()
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Game::from_id(model.id, &db, &cfg))
            .collect())
    }

    /// Search for a game by name
    // TODO: This is a bad name because you're just directly finding a game by its exact name. Not
    // exactly searching. We should have actual searching.
    pub(crate) async fn search(db: Db, cfg: Cfg, name: &str) -> Result<Option<Game>, SearchError> {
        Ok(Entity::find_by_name(name)
            .one(db.conn())
            .await
            .map_err(|source| SearchError {
                name: name.to_string(),
                source,
            })?
            .map(|model| Game::from_id(model.id, &db, &cfg)))
    }

    /// Make this game the active one
    pub async fn activate(&self) -> Result<(), ActivateError> {
        state::set_active_game_id(self.db.conn(), Some(self.id))
            .await
            .map_err(ActivateError::SetActiveGameId)
    }

    pub async fn is_active(&self) -> Result<bool, IsActiveError> {
        let active_game = Self::active(&self.db, &self.cfg)
            .await
            .map_err(IsActiveError::Active)?;

        Ok(active_game.as_ref() == Some(self))
    }

    pub(crate) async fn active(db: &Db, cfg: &Cfg) -> Result<Option<Game>, ActiveError> {
        Ok(Self::resolve_active_id(db.conn())
            .await
            .map_err(ActiveError::Resolve)?
            .map(|id| Game::from_id(id, db, cfg)))
    }

    // Returns the active game ID, selecting a fallback if none is set.
    async fn resolve_active_id(
        conn: &impl ConnectionTrait,
    ) -> Result<Option<i32>, ResolveActiveIdError> {
        if let Some(id) = state::active_game_id(conn)
            .await
            .map_err(ResolveActiveIdError::ActiveGameId)?
        {
            // We already have an active game set
            return Ok(Some(id));
        }

        // Pick the oldest game as the fallback
        let fallback_id = Entity::find()
            .order_by_id_asc()
            .one(conn)
            .await
            .map_err(ResolveActiveIdError::FindFallbackGame)?
            .map(|game| game.id);

        state::set_active_game_id(conn, fallback_id)
            .await
            .map_err(ResolveActiveIdError::SetActiveGameId)?;

        Ok(fallback_id)
    }
}
