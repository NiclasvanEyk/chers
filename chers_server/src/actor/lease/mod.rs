mod local;
pub use local::LocalLease;

#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "redis")]
pub use redis::RedisProvider;

#[cfg(feature = "nats")]
mod nats;
#[cfg(feature = "nats")]
pub use nats::NatsProvider;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::room::RoomId;

/// An enum dispatching to the available [`Provider`] implementations.
#[enum_dispatch::enum_dispatch(Provider)]
pub enum AnyLease {
    Local(LocalLease),
    #[cfg(feature = "redis")]
    Redis(RedisProvider),
    #[cfg(feature = "nats")]
    Nats(NatsProvider),
}

// Manual impl because `enum_dispatch` doesn't support `impl Future` in trait methods.
impl Provider for AnyLease {
    async fn try_acquire(&self, room_id: &RoomId) -> Result<Lease, LeaseError> {
        match self {
            AnyLease::Local(lease) => lease.try_acquire(room_id).await,
            #[cfg(feature = "redis")]
            AnyLease::Redis(lease) => lease.try_acquire(room_id).await,
            #[cfg(feature = "nats")]
            AnyLease::Nats(lease) => lease.try_acquire(room_id).await,
        }
    }
}

/// Ensures at most one server holds the write lease for a given room.
///
/// The lease must be held _before_ spawning the actor, and the actor
/// must stop processing as soon as the lease is lost.
pub trait Provider: Send + Sync {
    /// Try to take exclusive ownership of `room_id`.
    ///
    /// Returns `Err(LeaseError::HeldByOther)` when another server currently holds the lease.
    fn try_acquire(
        &self,
        room_id: &RoomId,
    ) -> impl Future<Output = Result<Lease, LeaseError>> + Send;
}

/// While this object lives, the current server is responsible for the game in question.
///
/// Drop it → lease is released.
/// [`expired()`](Self::expired) resolves when the lease is lost (renewal failure, revocation).
pub struct Lease {
    releaser: Option<Box<dyn FnOnce() + Send>>,
    expired: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Lease {
    pub(crate) fn new(
        releaser: Option<Box<dyn FnOnce() + Send>>,
        expired: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Self {
        Self { releaser, expired }
    }

    /// Wait until the lease is revoked or lost.
    pub async fn expired(&mut self) {
        (&mut self.expired).await
    }

    /// Voluntarily yield the lease (same as dropping).
    pub fn release(mut self) {
        self.release_inner();
    }

    fn release_inner(&mut self) {
        if let Some(releaser) = self.releaser.take() {
            releaser();
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.release_inner();
    }
}

/// Configuration for distributed lease backends.
#[derive(Clone, Copy)]
pub struct LeaseConfig {
    /// How long a lease is valid without renewal.
    pub ttl: Duration,

    /// How often the renewal task runs.
    pub renewal_interval: Duration,
}

impl LeaseConfig {
    pub fn from_env() -> Self {
        Self {
            ttl: Duration::from_secs(
                std::env::var("CHERS_LEASE_TTL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
            ),
            renewal_interval: Duration::from_secs(
                std::env::var("CHERS_LEASE_RENEWAL_INTERVAL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            ),
        }
    }
}

impl Default for LeaseConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30),
            renewal_interval: Duration::from_secs(10),
        }
    }
}

#[derive(Debug)]
pub enum LeaseError {
    Backend(Box<dyn std::error::Error + Send + Sync>),
    HeldByOther,
}

impl std::fmt::Display for LeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backend(err) => write!(f, "lease backend error: {err}"),
            Self::HeldByOther => write!(f, "lease held by another server"),
        }
    }
}

impl std::error::Error for LeaseError {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeaseImpl {
    Local,
    #[cfg(feature = "redis")]
    Redis,
}
