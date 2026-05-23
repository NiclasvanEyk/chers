//! Events broadcast by room actors.
//!
//! Re-exported from the shared API crate for convenience.
//!
//! Events are published via the [`EventBus`](crate::communication::bus::EventBus) trait
//! and delivered to all subscribers (WebSocket handlers).

pub use chers_server_api::v2::events::*;
