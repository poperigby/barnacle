use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

pub(crate) mod model;

use crate::{
    DeployKind, Game,
    deployers::{self, ItemGroup},
    repository::{
        db::Db,
        item::group::{ActiveModel, COLUMN, Entity},
    },
};

pub(crate) async fn find_id_by_key(db: &Db, game: &Game, group: impl ItemGroup) -> i32 {
    Entity::find()
        .filter(COLUMN.game_id.eq(game.id()))
        .filter(COLUMN.key.eq(group.key()))
        .one(db.conn())
        .await
        .unwrap()
        .unwrap()
        .id
}

pub(crate) async fn add_groups_to_game(db: &Db, game: &Game) {
    let deploy_kind = game.deploy_kind().await.unwrap();
    let game_id = game.id();

    match deploy_kind {
        DeployKind::OpenMW => {
            for group in deployers::openmw::OpenMw::item_groups() {
                add(db, game_id, group).await;
            }
        }
        _ => (),
    };
}

async fn add(db: &Db, game_id: i32, group: impl ItemGroup) {
    let model = ActiveModel {
        name: Set(group.name().to_string()),
        key: Set(group.key().to_string()),
        game_id: Set(game_id),
        sortable: Set(group.sortable()),
        ..Default::default()
    };

    Entity::insert(model).exec(db.conn()).await.unwrap();
}
