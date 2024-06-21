mod auth;
mod chat;
mod request_id;
mod server_time;

pub use auth::verify_token;
pub use chat::verify_chat;

use request_id::set_request_id;
use server_time::ServerTimeLayer;

use axum::{middleware::from_fn, Router};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

pub(crate) const REQUEST_ID_HEADER: &str = "x-request-id";
pub(crate) const SERVER_TIME_HEADER: &str = "x-server-time";

pub(crate) fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Millis),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
