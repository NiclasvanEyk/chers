use std::env;
use tracing_subscriber::prelude::*;

use serde_json::Value as JsonValue;

#[cfg(feature = "otel")]
use opentelemetry::trace::TracerProvider;

pub struct TelemetryConfig {
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
    pub service_version: String,
    #[cfg(feature = "otel")]
    pub otel_traces_sampler_arg: f64,
}

impl TelemetryConfig {
    pub fn from_env() -> Self {
        Self {
            otlp_endpoint: Self::otlp_endpoint_from_env(),
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "chers-server".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            #[cfg(feature = "otel")]
            otel_traces_sampler_arg: Self::sampler_arg_from_env(),
        }
    }

    fn otlp_endpoint_from_env() -> Option<String> {
        #[cfg(feature = "otel")]
        {
            return env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        }
        #[cfg(not(feature = "otel"))]
        None
    }

    #[cfg(feature = "otel")]
    fn sampler_arg_from_env() -> f64 {
        env::var("OTEL_TRACES_SAMPLER_ARG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0)
    }

    pub fn mode(&self) -> TelemetryMode {
        #[cfg(feature = "otel")]
        if self.otlp_endpoint.is_some() {
            return TelemetryMode::Otel;
        }
        TelemetryMode::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TelemetryMode {
    None,
    #[cfg(feature = "otel")]
    Otel,
}

pub struct TelemetryGuards {
    #[cfg(feature = "otel")]
    pub tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    pub mode: TelemetryMode,
}

impl TelemetryGuards {
    #[cfg(feature = "otel")]
    pub fn get_tracer(&self, name: &'static str) -> opentelemetry_sdk::trace::SdkTracer {
        self.tracer_provider
            .as_ref()
            .expect("OTEL tracer provider not initialized")
            .tracer(name)
    }
}

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

pub fn init(config: TelemetryConfig) -> TelemetryGuards {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    #[cfg(feature = "otel")]
    if let Some(provider) = try_init_otel(&config) {
        let tracer = provider.tracer("chers-server");
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(env_filter)
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
        tracing::info!(mode = ?TelemetryMode::Otel, "Telemetry initialized");
        return TelemetryGuards {
            tracer_provider: Some(provider),
            mode: TelemetryMode::Otel,
        };
    }

    let mode = config.mode();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();
    tracing::info!(mode = ?mode, "Telemetry initialized");
    TelemetryGuards {
        mode,
        #[cfg(feature = "otel")]
        tracer_provider: None,
    }
}

#[cfg(feature = "otel")]
fn try_init_otel(config: &TelemetryConfig) -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry::global;
    use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
    use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};

    if config.otlp_endpoint.is_none() {
        return None;
    }

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            opentelemetry::KeyValue::new(SERVICE_NAME, config.service_name.clone()),
            opentelemetry::KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
        ])
        .build();

    let exporter = match env::var("OTEL_EXPORTER_OTLP_PROTOCOL")
        .as_deref()
        .unwrap_or("grpc")
    {
        "http/protobuf" => opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .expect("Failed to build OTLP span exporter"),
        _ => opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to build OTLP span exporter"),
    };

    let sampler = Sampler::TraceIdRatioBased(config.otel_traces_sampler_arg);
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .build();

    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    Some(provider)
}

#[cfg_attr(not(feature = "otel"), allow(unused_variables))]
pub fn shutdown(guards: TelemetryGuards) {
    #[cfg(feature = "otel")]
    if let Some(provider) = guards.tracer_provider {
        if let Err(e) = provider.shutdown() {
            tracing::error!(error = ?e, "Failed to shutdown OTEL tracer provider");
        }
    }
    tracing::info!("Telemetry shutdown complete");
}

#[cfg(feature = "otel")]
pub(crate) mod otel;
