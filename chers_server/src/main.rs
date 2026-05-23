use std::sync::Arc;

use another_chess_server::actor::registry::RoomRegistry;
use another_chess_server::telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guards = telemetry::init(telemetry::TelemetryConfig::from_env());

    let event_bus = Arc::new(another_chess_server::config::event_bus().await?);
    let lease = Arc::new(another_chess_server::config::lease_provider().await?);
    let storage = Arc::new(another_chess_server::config::storage().await?);
    let command_bus = Arc::new(another_chess_server::config::command_bus().await?);

    let registry = RoomRegistry::new(lease, storage, event_bus, command_bus);

    let addr = std::env::var("CHERS_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
    another_chess_server::server::run(registry, &addr).await?;

    telemetry::shutdown(_guards);

    Ok(())
}
