use std::sync::Arc;

use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use tower_http::cors::CorsLayer;

use crate::actor::lease::Provider;
use crate::actor::registry::RoomRegistry;
use crate::communication::bus::EventBus;
use crate::communication::command::CommandBus;
use crate::communication::event::Event;
use crate::room::storage::Storage;

pub mod ws;

pub struct AppState<P, S, B, T> {
    pub registry: RoomRegistry<P, S, B, T>,
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

    let app = Router::new()
        .route("/rooms/{room_id}/ws", get(ws_handler::<P, S, B, T>))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
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
