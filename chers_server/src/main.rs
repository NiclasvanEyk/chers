mod handlers;
mod matches;

use crate::matches::repository::MatchRepository;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

pub struct AppState {
    pub matches: MatchRepository,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Don't initialize tracing_subscriber - shuttle does this automatically
    
    let app = Router::new()
        .route("/health", get(handlers::health::check))
        .route(
            "/matches/new",
            post(handlers::matches::new::create_new_match),
        )
        .route(
            "/matches/{id}/play",
            get(handlers::matches::play::websocket_handler),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(Arc::new(AppState {
            matches: MatchRepository::default(),
        }));

    info!("🚀 Chers server starting up...");
    Ok(app.into())
}
