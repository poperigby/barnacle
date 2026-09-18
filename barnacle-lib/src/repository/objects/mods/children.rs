use crate::repository::objects::items::Item;

use super::*;

impl Mod {
    pub(crate) async fn items(&self) -> Vec<Item> {
        Item::list(&self.db, &self.cfg, self).await
    }
}
