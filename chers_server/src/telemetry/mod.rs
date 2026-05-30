#[cfg(feature = "otel")]
pub(crate) mod otel;

#[cfg(feature = "sentry")]
pub(crate) mod sentry_integration;

use std::env;
use std::fmt::Display;

use serde_json::Value as JsonValue;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

/// Recursively replace known sensitive fields with `"***"`.
pub(crate) fn redact_value(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            if map.contains_key("secret") {
                map.insert("secret".into(), JsonValue::String("***".into()));
            }
            for val in map.values_mut() {
                redact_value(val);
            }
        }
        JsonValue::Array(arr) => {
            for val in arr.iter_mut() {
                redact_value(val);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            service_name: "chers-server".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: env::var("CHERS_ENV").unwrap_or_else(|_| "production".to_string()),
        }
    }
}

pub enum TelemetryFeatureMode {
    /// Feature is not compiled
    Disabled,
    /// Feature code is compiled, but e.g. an important env var like SENTRY_DSN is missing
    InActive,
    /// Integration working and actively gathering telemetry
    Active,
}

impl Display for TelemetryFeatureMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryFeatureMode::Disabled => write!(f, "disabled"),
            TelemetryFeatureMode::InActive => write!(f, "inactive"),
            TelemetryFeatureMode::Active => write!(f, "active"),
        }
    }
}

pub struct Telemetry {
    pub config: Config,

    #[cfg(feature = "otel")]
    tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,

    #[cfg(feature = "sentry")]
    #[allow(dead_code)]
    sentry_guard: Option<sentry::ClientInitGuard>,
}

impl Telemetry {
    pub fn init_from_env() -> Telemetry {
        let config = Config::from_env();
        let env_filter = Telemetry::env_filter();

        #[cfg(feature = "otel")]
        let tracer_provider = self::otel::init(&config, env_filter.clone());

        #[cfg(not(feature = "otel"))]
        Telemetry::init_fallback_subscriber(env_filter);

        #[cfg(feature = "otel")]
        if tracer_provider.is_none() {
            Telemetry::init_fallback_subscriber(env_filter);
        }

        return Telemetry {
            config: config.clone(),
            #[cfg(feature = "otel")]
            tracer_provider,
            #[cfg(feature = "sentry")]
            sentry_guard: self::sentry_integration::init(&config),
        };
    }

    pub fn otel_feature_mode(&self) -> TelemetryFeatureMode {
        #[cfg(not(feature = "otel"))]
        return TelemetryFeatureMode::Disabled;
        #[cfg(feature = "otel")]
        {
            match self.tracer_provider {
                Some(_) => TelemetryFeatureMode::Active,
                None => TelemetryFeatureMode::Disabled,
            }
        }
    }

    pub fn sentry_feature_mode(&self) -> TelemetryFeatureMode {
        #[cfg(not(feature = "sentry"))]
        return TelemetryFeatureMode::Disabled;
        #[cfg(feature = "sentry")]
        {
            match self.sentry_guard {
                Some(_) => TelemetryFeatureMode::Active,
                None => TelemetryFeatureMode::Disabled,
            }
        }
    }

    pub fn log_level(&self) -> LevelFilter {
        tracing::level_filters::LevelFilter::current()
    }

    fn env_filter() -> EnvFilter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    }

    fn init_fallback_subscriber(env_filter: EnvFilter) {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(env_filter)
            .init();
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        #[cfg(feature = "otel")]
        if let Some(provider) = &self.tracer_provider {
            if let Err(e) = provider.shutdown() {
                tracing::error!(error = ?e, "Failed to shutdown OTEL tracer provider");
            }
        }
        tracing::info!("Telemetry shutdown complete");
    }
}
