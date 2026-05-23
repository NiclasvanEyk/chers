use crate::room::RoomId;

use super::{Lease, LeaseError, Provider};

/// Single-server in-memory lease.
///
/// This does not really follow the same rules as other persistent leases, which re-new their
/// leases in a set interval. Instead, it just never expires the leases, which is fine as there
/// will only ever be one server anyways, so one could debate if this is even necessary.
pub struct LocalLease;

impl LocalLease {
    // Totally unnecessary, but looks more right to me compared to a bare name.
    pub fn new() -> LocalLease {
        return LocalLease;
    }
}

impl Provider for LocalLease {
    async fn try_acquire(&self, _room_id: &RoomId) -> Result<Lease, LeaseError> {
        Ok(Lease::new(None, Box::pin(std::future::pending())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn room_id(s: &str) -> RoomId {
        s.to_string()
    }

    #[tokio::test]
    async fn try_acquire_returns_guard() {
        let lease = LocalLease::new();
        lease.try_acquire(&room_id("room-1")).await.unwrap();
    }

    #[tokio::test]
    async fn guard_never_expires() {
        let lease = LocalLease::new();
        let mut guard = lease.try_acquire(&room_id("room-1")).await.unwrap();

        tokio::time::timeout(Duration::from_millis(10), guard.expired())
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn concurrent_acquires_always_succeed() {
        let lease = LocalLease::new();
        let _guard1 = lease.try_acquire(&room_id("room-1")).await.unwrap();
        let _guard2 = lease.try_acquire(&room_id("room-1")).await.unwrap();
    }

    #[tokio::test]
    async fn guard_drop_does_not_panic() {
        let lease = LocalLease::new();
        let guard = lease.try_acquire(&room_id("room-1")).await.unwrap();
        drop(guard);
    }

    #[tokio::test]
    async fn guard_release_does_not_panic() {
        let lease = LocalLease::new();
        let guard = lease.try_acquire(&room_id("room-1")).await.unwrap();
        guard.release();
    }
}
