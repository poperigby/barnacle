use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};

pub(crate) mod model;

use crate::{
    DeployKind, Profile,
    deployers::{self, ItemGroupSpec},
    repository::{
        db::Db,
        item_lists::{ActiveModel, COLUMN, Entity},
    },
};

pub(crate) async fn find(db: &Db, profile: &Profile, group: &str) -> i32 {
    Entity::find()
        .filter(COLUMN.group.eq(group))
        .filter(COLUMN.profile_id.eq(profile.id))
        .one(db.conn())
        .await
        .unwrap()
        .unwrap()
        .id
}

async fn add(db: &Db, profile_id: i32, group: impl ItemGroupSpec) {
    let model = ActiveModel {
        name: Set(group.name().to_string()),
        group: Set(group.key().to_string()),
        profile_id: Set(profile_id),
        sortable: Set(group.ordered()),
        ..Default::default()
    };

    Entity::insert(model).exec(db.conn()).await.unwrap();
}
