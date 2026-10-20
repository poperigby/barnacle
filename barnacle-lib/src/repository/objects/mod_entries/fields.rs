use sea_orm::{ExprTrait, sea_query::Expr};

use super::*;

impl ModEntry {
    pub async fn name(&self) -> Result<String, NameError> {
        self.mod_()
            .await
            .map_err(NameError::Load)?
            .name()
            .await
            .map_err(NameError::Name)
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        self.mod_()
            .await
            .map_err(DirError::Load)?
            .dir()
            .await
            .map_err(DirError::Dir)
    }

    pub async fn enabled(&self) -> Result<bool, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("enabled", source))?
            .enabled)
    }

    pub async fn set_enabled(&self, value: bool) -> Result<(), SetEnabledError> {
        let mut active_model = self
            .active_model(self.db.conn())
            .await
            .map_err(SetEnabledError::Load)?;
        active_model.enabled.set_if_not_equals(value);
        active_model
            .update(self.db.conn())
            .await
            .map_err(SetEnabledError::Update)?;

        Ok(())
    }

    pub async fn position(&self) -> i32 {
        self.model(self.db.conn()).await.unwrap().position
    }

    pub(crate) async fn next_position(db: &Db, profile: &Profile) -> Result<i32, sea_orm::DbErr> {
        Ok(Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .order_by_desc(COLUMN.position)
            .one(db.conn())
            .await?
            .map_or(0, |e| e.position + 1))
    }

    pub async fn set_position(&self, new_position: i32) {
        let conn = self.db.conn();

        let existing = Entity::find()
            .filter(COLUMN.id.ne(self.id))
            .filter(COLUMN.position.eq(new_position))
            .one(conn)
            .await
            .unwrap();

        // To make space, increment each following entry's position before setting the new position
        if let Some(existing) = existing {
            Entity::update_many()
                .col_expr(COLUMN.position, Expr::col(COLUMN.position).add(1))
                .filter(COLUMN.position.gte(existing.position))
                .filter(COLUMN.id.eq(self.id))
                .exec(conn)
                .await
                .unwrap();
        };

        let mut active_model = self.active_model(conn).await.unwrap();
        active_model.position.set_if_not_equals(new_position);
        active_model.update(conn).await.unwrap();
    }

    pub async fn notes(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("notes", source))?
            .notes)
    }
}
