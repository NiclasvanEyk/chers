use std::sync::Arc;

use another_chess_server::actor::registry::RoomRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let event_bus = Arc::new(another_chess_server::config::event_bus().await?);
    let lease = Arc::new(another_chess_server::config::lease_provider().await?);
    let storage = Arc::new(another_chess_server::config::storage().await?);
    let command_bus = Arc::new(another_chess_server::config::command_bus().await?);

    let registry = RoomRegistry::new(lease, storage, event_bus, command_bus);

    let addr = std::env::var("CHERS_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
    another_chess_server::server::run(registry, &addr).await
}
