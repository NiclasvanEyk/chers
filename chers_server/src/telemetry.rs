use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use sentry::ClientInitGuard;
use std::env;

pub struct TelemetryConfig {
    pub sentry_dsn: Option<String>,
    pub sentry_environment: Option<String>,
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
    pub service_version: String,
    pub sentry_traces_sample_rate: f32,
    pub otel_traces_sampler_arg: f64,
}

impl TelemetryConfig {
    pub fn from_env() -> Self {
        Self {
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            sentry_environment: env::var("SENTRY_ENVIRONMENT").ok(),
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "chers-server".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            sentry_traces_sample_rate: env::var("SENTRY_TRACES_SAMPLE_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            otel_traces_sampler_arg: env::var("OTEL_TRACES_SAMPLER_ARG")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
        }
    }

    pub fn mode(&self) -> TelemetryMode {
        match (&self.sentry_dsn, &self.otlp_endpoint) {
            (Some(_), Some(_)) => TelemetryMode::Both,
            (Some(_), None) => TelemetryMode::Sentry,
            (None, Some(_)) => TelemetryMode::Otel,
            (None, None) => TelemetryMode::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TelemetryMode {
    None,
    Otel,
    Sentry,
    Both,
}

pub struct TelemetryGuards {
    pub sentry: Option<ClientInitGuard>,
    pub tracer_provider: Option<SdkTracerProvider>,
    pub mode: TelemetryMode,
}

impl TelemetryGuards {
    /// Get a tracer for the configured provider
    /// Panics if called before telemetry is initialized and no OTEL provider exists
    pub fn get_tracer(&self, name: &'static str) -> opentelemetry_sdk::trace::SdkTracer {
        self.tracer_provider
            .as_ref()
            .expect("OTEL tracer provider not initialized")
            .tracer(name)
    }
}

pub fn init(config: TelemetryConfig) -> TelemetryGuards {
    let mode = config.mode();

    // Initialize Sentry if needed
    let sentry_environment_for_logging = config.sentry_environment.clone();
    let sentry_guard = if mode == TelemetryMode::Sentry || mode == TelemetryMode::Both {
        let dsn = config
            .sentry_dsn
            .as_ref()
            .expect("SENTRY_DSN should be set");
        let guard = sentry::init((
            dsn.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: config.sentry_environment.map(|e| e.into()),
                send_default_pii: true,
                enable_logs: true,
                traces_sample_rate: config.sentry_traces_sample_rate,
                ..Default::default()
            },
        ));

        Some(guard)
    } else {
        None
    };

    // Initialize OTEL tracer provider
    let tracer_provider = if mode == TelemetryMode::Otel || mode == TelemetryMode::Both {
        // Build resource using Resource::builder with attributes
        let resource = opentelemetry_sdk::Resource::builder()
            .with_attributes(vec![
                opentelemetry::KeyValue::new(SERVICE_NAME, config.service_name.clone()),
                opentelemetry::KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
            ])
            .build();

        // Build span exporter - endpoint comes from OTEL_EXPORTER_OTLP_ENDPOINT env var
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to build OTLP span exporter");

        // Build tracer provider with sampler
        let sampler = Sampler::TraceIdRatioBased(config.otel_traces_sampler_arg);
        let mut provider_builder = SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .with_sampler(sampler);

        // Add Sentry span processor if both modes are enabled
        if mode == TelemetryMode::Both {
            provider_builder = provider_builder
                .with_span_processor(sentry_opentelemetry::SentrySpanProcessor::new());
        }

        let provider = provider_builder.build();

        // Set propagator
        if mode == TelemetryMode::Both || mode == TelemetryMode::Sentry {
            global::set_text_map_propagator(sentry_opentelemetry::SentryPropagator::new());
        } else {
            global::set_text_map_propagator(
                opentelemetry_sdk::propagation::TraceContextPropagator::new(),
            );
        }

        // Set as global tracer provider
        global::set_tracer_provider(provider.clone());

        Some(provider)
    } else {
        None
    };

    tracing::info!(
        mode = ?mode,
        sentry_enabled = sentry_guard.is_some(),
        sentry_environment = ?sentry_environment_for_logging,
        otel_enabled = tracer_provider.is_some(),
        otlp_endpoint = ?config.otlp_endpoint,
        sampler_ratio = config.otel_traces_sampler_arg,
        "Telemetry initialized"
    );

    TelemetryGuards {
        sentry: sentry_guard,
        tracer_provider,
        mode,
    }
}

/// Shutdown telemetry gracefully
pub fn shutdown(guards: TelemetryGuards) {
    if let Some(provider) = guards.tracer_provider {
        // Get the provider and shutdown
        // Note: Arc makes this tricky - we need to try_unwrap or just shutdown via global
        if let Err(e) = provider.shutdown() {
            tracing::error!(error = ?e, "Failed to shutdown OTEL tracer provider");
        }
    }

    tracing::info!("Telemetry shutdown complete");
}
