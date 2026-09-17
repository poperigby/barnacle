use sea_orm::{ActiveValue::Set, ConnectionTrait, DatabaseTransaction, EntityTrait};

use crate::{
    deployers::ItemGroupSpec,
    repository::{
        item_groups::Model,
        objects::item_groups::model::{ActiveModel, Entity},
    },
};

pub(crate) mod model;

#[cfg(test)]
mod tests;

pub(crate) async fn insert(txn: &DatabaseTransaction, game_id: i32, spec: impl ItemGroupSpec) {
    let model = ActiveModel {
        game_id: Set(game_id),
        key: Set(spec.key().to_string()),
        name: Set(spec.name().to_string()),
        ordered: Set(spec.ordered()),
        ..Default::default()
    };

    Entity::insert(model).exec(txn).await.unwrap();
}

pub(crate) async fn find_by_key(
    conn: &impl ConnectionTrait,
    game_id: i32,
    key: &str,
) -> Option<Model> {
    Some(
        Entity::find_by_key_per_game((game_id, key.to_string()))
            .one(conn)
            .await
            .unwrap()
            .unwrap(),
    )
}
