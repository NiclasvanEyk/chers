use std::sync::Arc;

use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::actor::lease::Provider;
use crate::actor::registry::RoomRegistry;
use crate::communication::bus::EventBus;
use crate::communication::command::CommandBus;
use crate::communication::event::Event;
use crate::room::RoomId;
use crate::room::storage::Storage;

pub mod ws;

pub struct AppState<P, S, B, T> {
    pub registry: RoomRegistry<P, S, B, T>,
}

/// Response for creating a new room.
#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub room_id: RoomId,
}

async fn health_check() -> &'static str {
    "OK"
}

pub async fn run<P, S, B, T>(
    registry: RoomRegistry<P, S, B, T>,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Provider + Send + Sync + 'static,
    S: Storage + 'static,
    B: EventBus<Item = Event> + Send + Sync + 'static,
    T: CommandBus<Cmd = crate::communication::command::Command> + 'static,
{
    let state = Arc::new(AppState { registry });

    #[allow(unused_mut)]
    let mut app = Router::new()
        .route("/health", get(health_check))
        .route("/rooms/new", post(create_room_handler::<P, S, B, T>))
        .route("/rooms/{room_id}/ws", get(ws_handler::<P, S, B, T>))
        .layer(CorsLayer::permissive())
        .with_state(state);

    #[cfg(feature = "sentry")]
    {
        app = crate::telemetry::sentry_integration::apply_middleware(app);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for POST /rooms/new - creates a new room and returns the room ID.
async fn create_room_handler<P, S, B, T>(
    _state: State<Arc<AppState<P, S, B, T>>>,
) -> impl IntoResponse
where
    P: Provider + Send + Sync + 'static,
    S: Storage + 'static,
    B: EventBus<Item = Event> + Send + Sync + 'static,
    T: CommandBus<Cmd = crate::communication::command::Command> + 'static,
{
    // Generate a new room ID (UUID)
    let room_id = uuid::Uuid::now_v7().to_string();

    // Optionally, we could pre-create the room in the registry here,
    // but lazy creation on first WebSocket connection works fine too.
    // The room will be created when the first client connects via WS.

    let response = CreateRoomResponse { room_id };
    axum::Json(response)
}

async fn ws_handler<P, S, B, T>(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState<P, S, B, T>>>,
) -> impl IntoResponse
where
    P: Provider + Send + Sync + 'static,
    S: Storage + 'static,
    B: EventBus<Item = Event> + Send + Sync + 'static,
    T: CommandBus<Cmd = crate::communication::command::Command> + 'static,
{
    ws.on_upgrade(move |socket| ws::handle_socket(socket, state, room_id))
}
