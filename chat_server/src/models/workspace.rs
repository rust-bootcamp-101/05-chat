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
    use std::path::Path;

    use crate::{CreateUser, User};

    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

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
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let _ = Workspace::create(&pool, "test", 0).await?;
        let ws = Workspace::find_by_name(&pool, "test").await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create(&pool, "test", 0).await?;
        assert_eq!(ws.name, "test");

        let input = CreateUser::new(&ws.name, "user1", "randomemail1@acc.org", "password");
        let user1 = User::create(&pool, &input).await?;

        let input = CreateUser::new(&ws.name, "user2", "randomemail2@acc.org", "password");
        let user2 = User::create(&pool, &input).await?;

        let input = CreateUser::new(&ws.name, "user3", "randomemail3@acc.org", "password");
        let user3 = User::create(&pool, &input).await?;

        let users = Workspace::fetch_all_chat_users(&pool, ws.id as _).await?;
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[0].fullname, user1.fullname);
        assert_eq!(users[0].email, user1.email);

        assert_eq!(users[1].id, user2.id);
        assert_eq!(users[1].fullname, user2.fullname);
        assert_eq!(users[1].email, user2.email);

        assert_eq!(users[2].id, user3.id);
        assert_eq!(users[2].fullname, user3.fullname);
        assert_eq!(users[2].email, user3.email);
        Ok(())
    }
}
