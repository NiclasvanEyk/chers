mod handlers;
mod matches;
mod telemetry;

#[cfg(feature = "bundle-frontend")]
mod static_files;

use crate::matches::repository::MatchRepository;
use axum::routing::{get, post};
use axum::Router;
use axum::{body::Body, http::Request};
use std::sync::Arc;
use std::io;
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::prelude::*;

pub struct AppState {
    pub matches: MatchRepository,
}

fn main() -> io::Result<()> {
    // Load environment variables from .env file if present
    // Supports running from repo root or chers_server/ directory
    if let Ok(env_file) = std::env::var("ENV_FILE") {
        dotenvy::from_path(&env_file).ok();
    } else {
        // Try to load from chers_server/.env (when running from repo root)
        dotenvy::from_path(std::path::Path::new("chers_server").join(".env")).ok();
        // Or from current directory (when running from chers_server/)
        dotenvy::dotenv().ok();
    }

    // Load telemetry config (initialization happens inside async block where Tokio runtime exists)
    let config = telemetry::TelemetryConfig::from_env();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listen_address = std::env::var("CHERS_LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = std::env::var("CHERS_HOST").unwrap_or_else(|_| "localhost".to_string());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            // Initialize telemetry (requires Tokio runtime for batch exporter)
            let guards = telemetry::init(config);

            // Initialize tracing subscriber with appropriate layers based on telemetry mode
            // Default to INFO level if RUST_LOG is not set
            let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
            
            let subscriber = tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(env_filter);

            match guards.mode {
                telemetry::TelemetryMode::None => {
                    subscriber.init();
                }
                telemetry::TelemetryMode::Sentry => {
                    subscriber
                        .with(sentry::integrations::tracing::layer())
                        .init();
                }
                telemetry::TelemetryMode::Otel => {
                    let tracer = guards.get_tracer("chers-server");
                    subscriber
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .init();
                }
                telemetry::TelemetryMode::Both => {
                    let tracer = guards.get_tracer("chers-server");
                    subscriber
                        .with(sentry::integrations::tracing::layer())
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .init();
                }
            }

            info!(mode = ?guards.mode, "Telemetry initialized");

            // Verify embedded frontend files when bundle-frontend feature is enabled
            #[cfg(feature = "bundle-frontend")]
            {
                static_files::verify_embedded_files();
                info!("Frontend files verified and embedded");
            }

            // Build router with Sentry middleware only if Sentry is enabled
            let mut app = Router::new()
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

            // Add Sentry middleware layers if Sentry is enabled
            if guards.sentry.is_some() {
                app = app
                    .layer(NewSentryLayer::<Request<Body>>::new_from_top())
                    .layer(SentryHttpLayer::new().enable_transaction());
            }

            // Add static file routes when bundle-frontend feature is enabled
            // These must be added last as they include a catch-all route
            #[cfg(feature = "bundle-frontend")]
            {
                app = app
                    .route("/", get(static_files::serve_shell))
                    .route("/_shell.html", get(static_files::serve_shell))
                    .route("/{*path}", get(static_files::handler));
            }

            let listener = tokio::net::TcpListener::bind(format!("{}:{}", listen_address, port))
                .await
                .unwrap();
            
            // Helper to check if address is a wildcard (binds to all interfaces)
            let is_wildcard = |addr: &str| -> bool {
                matches!(addr, "0.0.0.0" | "::" | "[::]" | "*")
            };
            
            info!("🚀 Chers server ready");
            
            // Always show the clickable local URL
            let local_url = format!("http://{}:{}/", host, port);
            info!("   Local:   {}", local_url);
            
            // Only show "Network" line if listening on all interfaces
            if is_wildcard(&listen_address) {
                info!("   Network: http://{}:{}/ (all interfaces)", listen_address, port);
            }
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
            
            // Shutdown telemetry gracefully
            telemetry::shutdown(guards);
        });
    
    Ok(())
}
