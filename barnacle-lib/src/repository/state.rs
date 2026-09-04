use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::warn;

use crate::repository::{
    db::models::{
        games::Entity as GameEntity,
        profiles::{self, Entity as ProfileEntity},
        state::{ActiveModel, Entity, Model},
    },
    handles::Result,
};

const STATE_ROW_ID: i32 = 0;

pub(crate) async fn active_game_id(conn: &DatabaseConnection) -> Result<Option<i32>> {
    Ok(model(conn).await?.active_game_id)
}

pub(crate) async fn set_active_game_id(
    conn: &DatabaseConnection,
    game_id: Option<i32>,
) -> Result<()> {
    let mut active_model = active_model(conn).await?;
    active_model.active_game_id.set_if_not_equals(game_id);
    active_model.update(conn).await?;

    reconcile(conn).await?;

    Ok(())
}

pub(crate) async fn active_profile_id(conn: &DatabaseConnection) -> Result<Option<i32>> {
    Ok(model(conn).await?.active_profile_id)
}

pub(crate) async fn set_active_profile_id(
    conn: &DatabaseConnection,
    profile_id: Option<i32>,
) -> Result<()> {
    let mut active_model = active_model(conn).await?;
    active_model.active_profile_id.set_if_not_equals(profile_id);
    active_model.update(conn).await?;

    reconcile(conn).await?;

    Ok(())
}

/// Enforce and fix consistency of the active state.
pub(crate) async fn reconcile(conn: &DatabaseConnection) -> Result<()> {
    let game = match active_game_id(conn).await? {
        Some(id) => GameEntity::find_by_id(id).one(conn).await?,
        None => None,
    };
    let profile = match active_profile_id(conn).await? {
        Some(id) => ProfileEntity::find_by_id(id).one(conn).await?,
        None => None,
    };

    match (game, profile) {
        (Some(game), Some(profile)) => {
            if profile.game_id == game.id {
                return Ok(());
            };

            activate_first_profile_from_game(conn, game.id).await?;
        }
        (Some(game), None) => {
            activate_first_profile_from_game(conn, game.id).await?;
        }
        (None, Some(_)) => {
            // This should never happen without a bug, but we can still try to handle it.
            warn!("Active profile was set without an active game. Reconciling active state.");
            activate_first_game_and_profile(conn).await?;
        }
        (None, None) => {
            activate_first_game_and_profile(conn).await?;
        }
    }

    Ok(())
}

async fn model(conn: &DatabaseConnection) -> Result<Model> {
    match Entity::find_by_id(STATE_ROW_ID).one(conn).await? {
        Some(model) => Ok(model),
        None => {
            let state = ActiveModel {
                id: Set(STATE_ROW_ID),
                ..Default::default()
            };

            Ok(state.insert(conn).await?)
        }
    }
}

async fn active_model(conn: &DatabaseConnection) -> Result<ActiveModel> {
    Ok(model(conn).await?.into())
}

async fn activate_first_game_and_profile(conn: &DatabaseConnection) -> Result<()> {
    let fallback_game_id = GameEntity::find()
        .order_by_id_desc()
        .one(conn)
        .await?
        .map(|game| game.id);
    let fallback_profile_id = match fallback_game_id {
        Some(game_id) => fallback_profile_id(conn, game_id).await?,
        None => None,
    };

    let mut state = active_model(conn).await?;
    state.active_game_id = Set(fallback_game_id);
    state.active_profile_id = Set(fallback_profile_id);
    state.update(conn).await?;

    Ok(())
}

async fn activate_first_profile_from_game(conn: &DatabaseConnection, game_id: i32) -> Result<()> {
    let fallback_profile_id = fallback_profile_id(conn, game_id).await?;

    let mut state = active_model(conn).await?;
    state.active_profile_id = Set(fallback_profile_id);
    state.update(conn).await?;

    Ok(())
}

async fn fallback_profile_id(conn: &DatabaseConnection, game_id: i32) -> Result<Option<i32>> {
    Ok(ProfileEntity::find()
        .filter(profiles::COLUMN.game_id.eq(game_id))
        .order_by_id_desc()
        .one(conn)
        .await?
        .map(|p| p.id))
}
