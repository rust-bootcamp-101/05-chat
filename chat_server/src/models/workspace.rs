use sqlx::PgPool;

use crate::{AppError, ChatUser, Workspace};

impl Workspace {
    pub async fn create(pool: &PgPool, name: &str, user_id: u64) -> Result<Self, AppError> {
        let workspace = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(workspace)
    }

    pub async fn update_owner(&self, pool: &PgPool, owner_id: u64) -> Result<Self, AppError> {
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

    pub async fn find_by_id(pool: &PgPool, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }

    pub async fn fetch_all_chat_users(
        pool: &PgPool,
        ws_id: u64,
    ) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email FROM users WHERE ws_id = $1 ORDER BY id ASC
        "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_util::get_test_pool, CreateUser, User};

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::create(&pool, "test", 0).await?;
        assert_eq!(ws.name, "test");

        let input = CreateUser::new(&ws.name, "user1", "randomemail@acc.org", "password");
        let user = User::create(&pool, &input).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);
        assert!(user.password_hash.is_none());

        assert_eq!(user.ws_id, ws.id);
        let ws = ws.update_owner(&pool, user.id as u64).await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::find_by_name(&pool, "acme").await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let users = Workspace::fetch_all_chat_users(&pool, 1).await?;
        assert_eq!(users.len(), 5);

        Ok(())
    }
}
