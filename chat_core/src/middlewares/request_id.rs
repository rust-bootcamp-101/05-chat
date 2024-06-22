use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

use super::REQUEST_ID_HEADER;

/// 中间件，在request和response添加request id
pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id exists, do nothing, otherwise generate a new one
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            // 相比于 uuid v4，v7生成的uuid是有序的
            let request_id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };
    let mut res = next.run(req).await;
    let Some(id) = id else { return res };
    res.headers_mut().insert(REQUEST_ID_HEADER, id);
    res
}
