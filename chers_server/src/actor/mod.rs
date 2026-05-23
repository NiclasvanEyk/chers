/// Coordination primitives for the actor model.
pub mod lease;

/// Room actor lifecycle and command dispatch.
pub mod registry;

/// Main event loop for room actors.
pub mod loop_;

/// Actual logic for handling commands.
pub mod handler;

/// Room proxy — the public-facing handle for interacting with rooms.
pub mod proxy;

pub use proxy::Room;
pub use registry::RoomRegistry;
