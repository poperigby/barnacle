use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter};

pub(crate) mod model;

use crate::{
    DeployKind, Profile,
    deployers::{self, ItemGroup},
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

pub(crate) async fn add_all_to_profile(db: &Db, profile: &Profile) {
    let deploy_kind = profile.parent().await.unwrap().deploy_kind().await.unwrap();

    match deploy_kind {
        DeployKind::OpenMW => {
            for group in deployers::openmw::OpenMw::item_groups() {
                dbg!(group);
                add(db, profile.id, group).await;
            }
        }
        _ => (),
    };
}

async fn add(db: &Db, profile_id: i32, group: impl ItemGroup) {
    let model = ActiveModel {
        name: Set(group.name().to_string()),
        group: Set(group.key().to_string()),
        profile_id: Set(profile_id),
        sortable: Set(group.sortable()),
        ..Default::default()
    };

    Entity::insert(model).exec(db.conn()).await.unwrap();
}
