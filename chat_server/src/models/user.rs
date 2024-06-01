use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;

use crate::{AppError, User};

impl User {
    /// Find a user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }

    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        email: &str,
        fullname: &str,
        password: &str,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let user = sqlx::query_as(
            r#"
            INSERT INTO users (email, fullname, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, fullname, email, created_at
        "#,
        )
        .bind(email)
        .bind(fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    /// Verify email and password
    pub async fn verify(
        pool: &PgPool,
        email: &str,
        password: &str,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        let Some(mut user) = user else {
            return Ok(None);
        };
        // mem::take(&mut user.password_hash) 从user中取走password_hash的值，并置换为None，
        // 因为后续还要返回user，要保留所有权，取出password_hash的所有权用于比较密码是否正确
        let password_hash = mem::take(&mut user.password_hash);
        let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;
        if !is_valid {
            return Ok(None);
        }
        Ok(Some(user))
    }
}

/// 加密密码
fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

/// 验证密码
fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[test]
    fn hash_password_and_verify_should_word() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        let ret = verify_password(password, &password_hash)?;
        assert_eq!(password_hash.len(), 97);
        assert!(ret);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let email = "randomemail@acc.org";
        let name = "user1";
        let password = "password";
        let user = User::create(&pool, email, name, password).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);
        assert!(user.password_hash.is_none());

        let user = User::find_by_email(&pool, email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.password_hash.is_none());

        let user = User::verify(&pool, email, password).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.password_hash.is_none());
        Ok(())
    }
}
