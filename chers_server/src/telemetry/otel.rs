use std::env;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use serde_json::Value as JsonValue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::communication::command::Command;
use crate::communication::event::Event;
use crate::telemetry::redact_value;

pub(crate) struct Config {
    pub otlp_endpoint: Option<String>,
}

impl Config {
    pub(crate) fn from_env() -> Config {
        Config {
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        }
    }
}

/// Recursively flatten a JSON value into dot-separated key-value pairs.
///
/// Objects produce `prefix.key` entries. Arrays produce `prefix.[index]` entries.
fn flatten_json(value: &JsonValue, prefix: &str) -> Vec<(String, String)> {
    match value {
        JsonValue::Object(map) => {
            let mut result = Vec::new();
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                result.extend(flatten_json(val, &new_prefix));
            }
            result
        }
        JsonValue::Array(arr) => {
            let mut result = Vec::new();
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}.[{}]", prefix, i);
                result.extend(flatten_json(val, &new_prefix));
            }
            result
        }
        JsonValue::String(s) => {
            vec![(prefix.to_string(), s.clone())]
        }
        JsonValue::Number(n) => {
            vec![(prefix.to_string(), n.to_string())]
        }
        JsonValue::Bool(b) => {
            vec![(prefix.to_string(), b.to_string())]
        }
        JsonValue::Null => {
            vec![(prefix.to_string(), "null".to_string())]
        }
    }
}

/// Extract the outer enum variant chain as a "type" string and the inner payload.
///
/// For a 2‑level enum (e.g. `Event::Lobby(LobbyEvent::PlayerJoined { .. })`):
///   Input: `{"Lobby": {"PlayerJoined": {"user": {...}}}}`
///   Output: `("Lobby:PlayerJoined", {"user": {...}})`
///
/// For a 1‑level struct variant (e.g. `Command::RequestState { user }`):
///   Input: `{"RequestState": {"user": {...}}}`
///   Output: `("RequestState", {"user": {...}})`
///
/// Enum variants are identified by PascalCase keys (first character uppercase),
/// struct fields by snake_case keys (first character lowercase).
fn extract_type_and_payload(value: &mut JsonValue) -> (String, JsonValue) {
    let JsonValue::Object(map) = value else {
        return ("unknown".into(), value.take());
    };

    let first_key = match map.keys().next().cloned() {
        Some(k) => k,
        None => return ("unknown".into(), JsonValue::Null),
    };

    let first_val = map.remove(first_key.as_str()).unwrap_or(JsonValue::Null);

    if let JsonValue::Object(mut inner_map) = first_val {
        if let Some(inner_key) = inner_map.keys().next().cloned() {
            if inner_key.chars().next().map_or(false, |c| c.is_uppercase()) {
                let type_name = format!("{}:{}", first_key, inner_key);
                let payload = inner_map
                    .remove(inner_key.as_str())
                    .unwrap_or(JsonValue::Null);
                return (type_name, payload);
            }
        }
        return (first_key, JsonValue::Object(inner_map));
    }

    (first_key, first_val)
}

/// Add structured OTel attributes for an event span.
///
/// Sets `event.type` (e.g., `"Lobby:PlayerJoined"`) and flattens the payload into
/// `event.payload.*` keys.
pub(crate) fn add_event_attributes(span: &tracing::Span, event: &Event) {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let mut json = serde_json::to_value(event).unwrap_or_default();
    redact_value(&mut json);

    let (type_name, payload) = extract_type_and_payload(&mut json);
    let ctx = span.context();
    let otel_span = ctx.span();

    otel_span.set_attribute(opentelemetry::KeyValue::new("event.type", type_name));

    for (key, value) in flatten_json(&payload, "event.payload") {
        otel_span.set_attribute(opentelemetry::KeyValue::new(key, value));
    }
}

/// Add structured OTel attributes for a command span.
///
/// Sets `command.type` (e.g., `"Lobby:Join"`) and flattens the payload into
/// `command.payload.*` keys.
pub(crate) fn add_command_attributes(span: &tracing::Span, cmd: &Command) {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let mut json = serde_json::to_value(cmd).unwrap_or_default();
    redact_value(&mut json);

    let (type_name, payload) = extract_type_and_payload(&mut json);
    let ctx = span.context();
    let otel_span = ctx.span();

    otel_span.set_attribute(opentelemetry::KeyValue::new("command.type", type_name));

    for (key, value) in flatten_json(&payload, "command.payload") {
        otel_span.set_attribute(opentelemetry::KeyValue::new(key, value));
    }
}

pub(crate) fn init(
    chers_config: &super::Config,
    env_filter: EnvFilter,
) -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    let otel_config = Config::from_env();
    if otel_config.otlp_endpoint.is_none() {
        return None;
    }

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            opentelemetry::KeyValue::new(SERVICE_NAME, chers_config.service_name.clone()),
            opentelemetry::KeyValue::new(SERVICE_VERSION, chers_config.service_version.clone()),
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

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer(chers_config.service_name.clone());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Some(provider)
}
