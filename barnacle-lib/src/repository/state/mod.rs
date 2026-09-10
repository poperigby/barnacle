use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};

use crate::repository::{
    db::models::state::{ActiveModel, Entity, Model},
    state::error::{LoadStateError, SetActiveGameIdError},
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
