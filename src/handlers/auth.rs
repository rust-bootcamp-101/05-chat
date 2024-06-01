use axum::response::IntoResponse;

pub(crate) async fn singin_handler() -> impl IntoResponse {
    "singin"
}

pub(crate) async fn singout_handler() -> impl IntoResponse {
    "singout"
}
