mod api;
mod communication;
mod room;

use crate::room::repository::InMemoryRoomRepository;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub struct AppState {
    pub matches: Mutex<InMemoryRoomRepository>,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let spans = DefaultMakeSpan::default().include_headers(false);
    let tracing = TraceLayer::new_for_http().make_span_with(spans);
    let cors = CorsLayer::new().allow_origin(Any);

    let matches = Mutex::new(InMemoryRoomRepository::default());
    let app_state = Arc::new(AppState { matches });

    let app = Router::new()
        .route("/health", get(api::health::check))
        .route("/matches/new", post(api::matches::new::create_new_match))
        .route(
            "/matches/:id/play",
            get(api::matches::play::websocket_handler),
        )
        .layer(tracing)
        .layer(cors)
        .with_state(app_state);

    Ok(app.into())
}
