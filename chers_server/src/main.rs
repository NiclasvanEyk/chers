use chers_server::{config, server, telemetry::Telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let telemetry = Telemetry::init_from_env();
    tracing::info!(
        sentry = %telemetry.sentry_feature_mode(),
        otel = %telemetry.otel_feature_mode(),
        logging = %telemetry.log_level(),
        "Telemetry initialized",
    );

    let registry = config::room_registry().await?;
    server::run(registry, &config::server_address()).await
}
