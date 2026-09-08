use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::warn;

use crate::repository::{
    db::models::{
        games::Entity as GameEntity,
        profiles::{self, Entity as ProfileEntity},
        state::{ActiveModel, Entity, Model},
    },
    state::error::{
        LoadStateError, ReconcileError, SelectFallbackGameError, SelectFallbackProfileError,
        SetActiveGameIdError, SetActiveProfileIdError,
    },
};

pub(crate) mod error;

const STATE_ROW_ID: i32 = 0;

pub(crate) async fn active_game_id(
    conn: &impl ConnectionTrait,
) -> Result<Option<i32>, LoadStateError> {
    Ok(model(conn).await?.active_game_id)
}

pub(crate) async fn set_active_game_id(
    conn: &impl ConnectionTrait,
    game_id: Option<i32>,
) -> Result<(), SetActiveGameIdError> {
    let mut active_model = active_model(conn)
        .await
        .map_err(SetActiveGameIdError::LoadState)?;

    active_model.active_game_id.set_if_not_equals(game_id);

    active_model
        .update(conn)
        .await
        .map_err(SetActiveGameIdError::Update)?;

    reconcile(conn)
        .await
        .map_err(SetActiveGameIdError::Reconcile)?;

    Ok(())
}

pub(crate) async fn active_profile_id(
    conn: &impl ConnectionTrait,
) -> Result<Option<i32>, LoadStateError> {
    Ok(model(conn).await?.active_profile_id)
}

pub(crate) async fn set_active_profile_id(
    conn: &impl ConnectionTrait,
    profile_id: Option<i32>,
) -> Result<(), SetActiveProfileIdError> {
    let mut active_model = active_model(conn)
        .await
        .map_err(SetActiveProfileIdError::LoadState)?;

    active_model.active_profile_id.set_if_not_equals(profile_id);

    active_model
        .update(conn)
        .await
        .map_err(SetActiveProfileIdError::Update)?;

    reconcile(conn)
        .await
        .map_err(SetActiveProfileIdError::Reconcile)?;

    Ok(())
}

/// Enforce and fix consistency of the active state.
pub(crate) async fn reconcile(conn: &impl ConnectionTrait) -> Result<(), ReconcileError> {
    let game = match active_game_id(conn)
        .await
        .map_err(ReconcileError::ActiveGameId)?
    {
        Some(id) => GameEntity::find_by_id(id)
            .one(conn)
            .await
            .map_err(|source| ReconcileError::LoadActiveGame { id, source })?,
        None => None,
    };
    let profile = match active_profile_id(conn)
        .await
        .map_err(ReconcileError::ActiveProfileId)?
    {
        Some(id) => ProfileEntity::find_by_id(id)
            .one(conn)
            .await
            .map_err(|source| ReconcileError::LoadActiveProfile { id, source })?,
        None => None,
    };

    match (game, profile) {
        (Some(game), Some(profile)) => {
            if profile.game_id == game.id {
                return Ok(());
            };

            select_fallback_profile(conn, game.id)
                .await
                .map_err(|source| ReconcileError::SelectFallbackProfile {
                    game_id: game.id,
                    source,
                })?;
        }
        (Some(game), None) => {
            select_fallback_profile(conn, game.id)
                .await
                .map_err(|source| ReconcileError::SelectFallbackProfile {
                    game_id: game.id,
                    source,
                })?;
        }
        (None, Some(_)) => {
            // This should never happen without a bug, but we can still try to handle it.
            warn!("Active profile was set without an active game. Reconciling active state.");
            let fallback_game_id = select_fallback_game(conn)
                .await
                .map_err(ReconcileError::SelectFallbackGame)?;

            if let Some(active_game_id) = fallback_game_id {
                select_fallback_profile(conn, active_game_id)
                    .await
                    .map_err(|source| ReconcileError::SelectFallbackProfile {
                        game_id: active_game_id,
                        source,
                    })?;
            }
        }
        (None, None) => {
            let fallback_game_id = select_fallback_game(conn)
                .await
                .map_err(ReconcileError::SelectFallbackGame)?;

            if let Some(active_game_id) = fallback_game_id {
                select_fallback_profile(conn, active_game_id)
                    .await
                    .map_err(|source| ReconcileError::SelectFallbackProfile {
                        game_id: active_game_id,
                        source,
                    })?;
            }
        }
    }

    Ok(())
}

async fn model(conn: &impl ConnectionTrait) -> Result<Model, LoadStateError> {
    match Entity::find_by_id(STATE_ROW_ID)
        .one(conn)
        .await
        .map_err(LoadStateError::Query)?
    {
        Some(model) => Ok(model),
        None => {
            let state = ActiveModel {
                id: Set(STATE_ROW_ID),
                ..Default::default()
            };

            Ok(state.insert(conn).await.map_err(LoadStateError::Create)?)
        }
    }
}

async fn active_model(conn: &impl ConnectionTrait) -> Result<ActiveModel, LoadStateError> {
    Ok(model(conn).await?.into())
}

async fn select_fallback_game(
    conn: &impl ConnectionTrait,
) -> Result<Option<i32>, SelectFallbackGameError> {
    let fallback_game_id = GameEntity::find()
        .order_by_id_desc()
        .one(conn)
        .await
        .map_err(SelectFallbackGameError::FindFallbackGame)?
        .map(|game| game.id);

    let mut state = active_model(conn)
        .await
        .map_err(SelectFallbackGameError::LoadState)?;

    state.active_game_id = Set(fallback_game_id);
    state
        .update(conn)
        .await
        .map_err(SelectFallbackGameError::Update)?;

    Ok(fallback_game_id)
}

async fn select_fallback_profile(
    conn: &impl ConnectionTrait,
    game_id: i32,
) -> Result<(), SelectFallbackProfileError> {
    let fallback_profile_id = ProfileEntity::find()
        .filter(profiles::COLUMN.game_id.eq(game_id))
        .order_by_id_desc()
        .one(conn)
        .await
        .map_err(|source| SelectFallbackProfileError::FindFallbackProfile { game_id, source })?
        .map(|p| p.id);

    let mut state = active_model(conn)
        .await
        .map_err(SelectFallbackProfileError::LoadState)?;

    state.active_profile_id = Set(fallback_profile_id);

    state
        .update(conn)
        .await
        .map_err(|source| SelectFallbackProfileError::Update { game_id, source })?;

    Ok(())
}
