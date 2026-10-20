use futures::TryFutureExt;
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

    pub async fn position(&self) -> Result<u32, GetFieldError> {
        let position = self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("position", source))?
            .position;

        Ok(u32::try_from(position).expect(
            "the conversion must work, because the database field is constrained to 0 through u32::MAX",
        ))
    }

    pub(crate) async fn next_position(db: &Db, profile: &Profile) -> Result<i64, sea_orm::DbErr> {
        Ok(Entity::find()
            .filter(COLUMN.profile_id.eq(profile.id))
            .order_by_desc(COLUMN.position)
            .one(db.conn())
            .await?
            .map_or(0, |e| e.position + 1))
    }

    pub async fn set_position(&self, new_position: u32) -> Result<(), SetPositionError> {
        let conn = self.db.conn();
        let profile_id = self.parent().await.map_err(SetPositionError::Parent)?.id;

        let old_position = self
            .position()
            .map_err(SetPositionError::OldPosition)
            .await?;

        if new_position == old_position {
            return Ok(());
        }

        // We calculate this before moving anything, because if we calculate it after we move the
        // target, the last position could already be empty, and then no longer considered the last
        // position. This means it will end up not getting filled, because our filter excludes it.
        let last_position = Entity::find()
            .order_by_desc(COLUMN.position)
            .filter(COLUMN.profile_id.eq(profile_id))
            .one(conn)
            .await
            .map_err(SetPositionError::LastPosition)?
            .map_or(0, |m| m.position);

        let mut active_model = self
            .active_model(conn)
            .await
            .map_err(SetPositionError::Load)?;
        active_model.position.set_if_not_equals(new_position.into());
        active_model
            .update(conn)
            .await
            .map_err(SetPositionError::Update)?;

        if new_position > old_position {
            Entity::update_many()
                .col_expr(COLUMN.position, Expr::col(COLUMN.position).sub(1))
                .filter(COLUMN.profile_id.eq(profile_id))
                .filter(COLUMN.position.lte(new_position))
                .filter(COLUMN.id.ne(self.id))
                .filter(COLUMN.position.ne(0))
                .exec(conn)
                .await
                .map_err(SetPositionError::UpdatePositions)?;
        } else if new_position < old_position {
            Entity::update_many()
                .col_expr(COLUMN.position, Expr::col(COLUMN.position).add(1))
                .filter(COLUMN.profile_id.eq(profile_id))
                .filter(COLUMN.position.gte(new_position))
                .filter(COLUMN.id.ne(self.id))
                .filter(COLUMN.position.ne(last_position))
                .exec(conn)
                .await
                .map_err(SetPositionError::UpdatePositions)?;
        }

        Ok(())
    }

    pub async fn notes(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("notes", source))?
            .notes)
    }
}

#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use rstest::rstest;

    use crate::{Repository, games::Kind};

    #[rstest]
    #[case::first_to_last("A", 3, ["B", "C", "D", "A"])]
    #[case::last_to_first("D", 0, ["D", "A", "B", "C"])]
    #[case::forward_one("B", 2, ["A", "C", "B", "D"])]
    #[case::backward_one("C", 1, ["A", "C", "B", "D"])]
    #[case::same_position("B", 1, ["A", "B", "C", "D"])]
    #[tokio::test]
    async fn set_position(#[case] name: &str, #[case] to: u32, #[case] expected: [&str; 4]) {
        let repo = Repository::in_memory().await;
        let game = repo.add_game("Morrowind", Kind::OpenMW).await.unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        let mod_a = game.new_mod("A").empty().await;
        let a = profile.add_mod_entry(mod_a).await.unwrap();
        let mod_b = game.new_mod("B").empty().await;
        let b = profile.add_mod_entry(mod_b).await.unwrap();
        let mod_c = game.new_mod("C").empty().await;
        let c = profile.add_mod_entry(mod_c).await.unwrap();
        let mod_d = game.new_mod("D").empty().await;
        let d = profile.add_mod_entry(mod_d).await.unwrap();

        let entry = match name {
            "A" => &a,
            "B" => &b,
            "C" => &c,
            "D" => &d,
            _ => panic!("Unknown test entry: {name}"),
        };
        entry.set_position(to).await.unwrap();

        let entries = profile.mod_entries().await.unwrap();
        let actual = join_all(
            entries
                .iter()
                .map(async |entry| (entry.name().await.unwrap(), entry.position().await.unwrap())),
        )
        .await;

        let expected: Vec<_> = expected
            .into_iter()
            .enumerate()
            .map(|(position, name)| (name.to_owned(), u32::try_from(position).unwrap()))
            .collect();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn set_position_single_entry() {
        let repo = Repository::in_memory().await;
        let game = repo.add_game("Morrowind", Kind::OpenMW).await.unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        let mod_a = game.new_mod("A").empty().await;
        let a = profile.add_mod_entry(mod_a).await.unwrap();

        assert_eq!(0, a.position().await.unwrap());

        a.set_position(0).await.unwrap();

        assert_eq!(0, a.position().await.unwrap());
    }

    #[tokio::test]
    async fn set_position_beyond_end() {
        let repo = Repository::in_memory().await;
        let game = repo.add_game("Morrowind", Kind::OpenMW).await.unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        let mod_a = game.new_mod("A").empty().await;
        let a = profile.add_mod_entry(mod_a).await.unwrap();

        a.set_position(100).await.unwrap();
    }
}
