mod handlers;
mod matches;

use crate::matches::repository::MatchRepository;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub matches: MatchRepository,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with env filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

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

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("🚀 Chers server starting on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
