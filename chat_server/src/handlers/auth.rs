use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrorOutput,
    models::{CreateUser, SigninUser},
    AppError, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(input).await?;
    let token = state.ek.sign(user)?;
    // let mut header = HeaderMap::new();
    // header.insert("X-Token", token.parse()?);
    // Ok((StatusCode::CREATED, header))
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

// TODO: 重复登录产生多个token如何处理？(1.采用类似session的机制，强制取消上一个token; 2.多端登录(设备唯一表示))
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(&input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            let body = Json(AuthOutput { token });
            Ok((StatusCode::OK, body).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password".to_string()));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "user1", "randomemail@acc.org", "password");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert!(!ret.token.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Startdusk Shelby", "startdusk@acme.org", "password");
        let ret = signup_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "email already exists: startdusk@acme.org");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "startdusk@acme.org";
        let password = "password";

        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert!(!ret.token.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exists_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "noneuser@acc.org";
        let password = "password";

        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "Invalid email or password");
        Ok(())
    }
}
