use std::sync::Arc;

use chers_server::actor::registry::RoomRegistry;
use chers_server::telemetry::Telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry = Telemetry::init_from_env();
    tracing::info!("Telemetry initialized"); // TODO: Bitflags of modes: Sentry / OTEL

    let event_bus = Arc::new(chers_server::config::event_bus().await?);
    let lease = Arc::new(chers_server::config::lease_provider().await?);
    let storage = Arc::new(chers_server::config::storage().await?);
    let command_bus = Arc::new(chers_server::config::command_bus().await?);

    let registry = RoomRegistry::new(lease, storage, event_bus, command_bus);

    let addr = std::env::var("PORT")
        .map(|p| format!("0.0.0.0:{p}"))
        .or_else(|_| std::env::var("CHERS_ADDR"))
        .unwrap_or_else(|_| "0.0.0.0:8000".into());
    chers_server::server::run(registry, &addr).await?;

    Ok(())
}
