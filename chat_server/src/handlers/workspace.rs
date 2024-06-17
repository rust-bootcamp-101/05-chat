use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{AppError, AppState, User, Workspace};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = Workspace::fetch_all_chat_users(&state.pool, user.ws_id as _).await?;

    Ok(Json(users))
}
