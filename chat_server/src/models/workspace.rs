use sqlx::PgPool;

use crate::{AppError, AppState, Workspace};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(workspace)
    }

    #[allow(unused)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
}

impl Workspace {
    pub async fn update_owner(&self, pool: &PgPool, owner_id: u64) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            UPDATE workspaces SET owner_id = $1
            WHERE id = $2 AND (SELECT ws_id FROM users WHERE id = $1) = $2
            RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {
    use crate::CreateUser;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;
        assert_eq!(ws.name, "test");

        let input = CreateUser::new(&ws.name, "user1", "randomemail@acc.org", "password");
        let user = state.create_user(input.clone()).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);
        assert!(user.password_hash.is_none());

        assert_eq!(user.ws_id, ws.id);
        let ws = ws.update_owner(&state.pool, user.id as _).await?;

        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("acme").await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let users = state.fetch_chat_users(1).await?;
        assert_eq!(users.len(), 5);

        Ok(())
    }
}
