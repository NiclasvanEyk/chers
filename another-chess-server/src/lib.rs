/// Message exchange between components.
pub mod communication;

/// Coordination primitives for the actor model (leases, registry, etc.).
pub mod actor;

/// Player connection and auth{orization,entication} management.
pub mod auth;

/// Phase-independent container for players, spectators, etc.
pub mod room;

/// Actually building components from env vars.
pub mod config;

/// Shared Redis connection utilities.
#[cfg(feature = "redis")]
pub mod redis;

/// HTTP + WebSocket server.
pub mod server;

pub mod utils {
    pub type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
}
