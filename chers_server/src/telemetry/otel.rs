use serde_json::Value as JsonValue;

use crate::communication::command::Command;
use crate::communication::event::Event;
use crate::telemetry::redact_value;

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
