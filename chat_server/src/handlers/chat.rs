use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{AppError, AppState, CreateChat, UpdateChat};
use chat_core::User;

#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = [Chat])
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chats(user.id as _, user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(chats)))
}

#[utoipa::path(
    post,
    path = "/api/chats",
    responses(
        (status = 201, description = "Create chat", body = Chat)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    // check user should in members
    if !input.members.contains(&user.id) {
        return Err(AppError::CreateChatError(format!(
            "user {} not in chat members {:?}",
            user.id, input.members
        )));
    }
    let chat = state
        .create_chat(input, user.id as _, user.ws_id as _)
        .await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 201, description = "Chat found", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}

// TODO: chats表需要添加一个owner(外键 user id)字段
// 检查是否能改名（owner可以改？）
// 可以删除members?
// owner退出members了怎么办
// members只剩一个人怎么办
pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    let Some(mut chat) = chat else {
        return Err(AppError::NotFound(format!("chat id {id}")));
    };

    if let Some(name) = input.name {
        chat.name = Some(name)
    }

    if let Some(members) = input.members {
        if !members.contains(&user.id) {
            // 如果更新人不在members里面了?
        }
        chat.members = members
    }

    let chat = state.update_chat(id, chat).await?;

    Ok((StatusCode::OK, Json(chat)))
}

// TODO: 下面连个接口需要存在设计上的缺陷，需要确定谁能删除
pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat"
}
