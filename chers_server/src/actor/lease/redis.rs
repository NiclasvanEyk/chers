use std::time::Duration;

use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tokio::sync::oneshot;
use tracing::{Instrument, error, info, warn};

use super::{Lease, LeaseConfig, LeaseError, Provider};
use crate::room::RoomId;

const KEY_PREFIX: &str = "chers:lease:";

/// Distributed lease provider backed by Redis.
///
/// Uses `SET … NX PX` for atomic acquisition and a Lua script
/// for safe renewal with instance-ownership checks.
///
/// When the renewal task detects that the key was taken over by
/// another instance (e.g. during a network partition) or when
/// the Redis connection fails, the [`Lease::expired`] future
/// resolves and the actor should stop processing.
pub struct RedisProvider {
    conn: ConnectionManager,
    config: LeaseConfig,
    instance_id: String,
}

impl RedisProvider {
    /// Create a provider from an existing Redis connection.
    ///
    /// `instance_id` – unique identifier for this server, used as the value
    /// stored in Redis to detect lease ownership during renewal.
    pub fn new(conn: ConnectionManager, instance_id: String, config: LeaseConfig) -> Self {
        info!(%instance_id, "Initialized Redis lease provider");
        Self {
            conn,
            config,
            instance_id,
        }
    }
}

impl Provider for RedisProvider {
    async fn try_acquire(&self, room_id: &RoomId) -> Result<Lease, LeaseError> {
        let key = format!("{KEY_PREFIX}{room_id}");
        // safety: Redis PX expects a 64-bit integer, and a TTL beyond ~584M years
        // is not going to happen, so a simple is fine.
        let ttl_ms = self.config.ttl.as_millis() as u64;

        let mut conn = self.conn.clone();

        let response = redis::cmd("SET")
            .arg(&key)
            .arg(&self.instance_id)
            .arg("NX")
            .arg("PX")
            .arg(ttl_ms)
            .query_async::<Option<String>>(&mut conn)
            .await;

        // Redis protocol: `SET … NX` returns OK (bulk string) when the
        // key was created, and nil when it already exists. The redis crate
        // maps nil → `None` (lease held by another server) and OK →
        // `Some("OK")` (we now hold it).  A connection or protocol error
        // means we can't tell either way — treat it as a backend failure.
        let result: Option<String> =
            response.map_err(|redis_error| LeaseError::Backend(redis_error.into()))?;

        let acquired = result.is_some();
        if !acquired {
            return Err(LeaseError::HeldByOther);
        }

        info!(%room_id, "Acquired lease");

        // Channels for coordinating with the renewal task
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        let (expired_tx, expired_rx) = oneshot::channel::<()>();

        let renew_key = key.clone();
        let renew_instance = self.instance_id.clone();
        let mut renew_conn = self.conn.clone();
        let ttl = self.config.ttl;
        let renew_interval = self.config.renewal_interval;

        let room_id = room_id.clone();
        let span_instance = renew_instance.clone();
        tokio::spawn(
            async move {
                let mut interval = tokio::time::interval(renew_interval);
                // Skip the first tick – the key was just set with a TTL
                interval.tick().await;

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            match renew_lease(&mut renew_conn, &renew_key, &renew_instance, ttl).await {
                                Ok(true) => {},
                                Ok(false) => {
                                    warn!(key = %renew_key, "Lease was lost to another instance");
                                    let _ = expired_tx.send(());
                                    return;
                                }
                                Err(e) => {
                                    error!(%e, "Lease renewal failed");
                                    let _ = expired_tx.send(());
                                    return;
                                }
                            }
                        }
                        _ = &mut cancel_rx => {
                            info!(key = %renew_key, "Releasing lease");
                            let _: Result<(), _> = renew_conn.del(&renew_key).await;
                            return;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("lease:renew", %room_id, %span_instance)),
        );

        let expired = async move {
            let _ = expired_rx.await;
        };

        Ok(Lease::new(
            Some(Box::new(move || {
                let _ = cancel_tx.send(());
            })),
            Box::pin(expired),
        ))
    }
}

/// Extend the TTL on a lease only if we still own it.
async fn renew_lease(
    conn: &mut ConnectionManager,
    key: &str,
    expected_value: &str,
    ttl: Duration,
) -> Result<bool, LeaseError> {
    let script = redis::Script::new(
        r#"
        if redis.call("GET", KEYS[1]) == ARGV[1] then
            redis.call("PEXPIRE", KEYS[1], ARGV[2])
            return 1
        else
            return 0
        end
        "#,
    );

    let result: bool = script
        .key(key)
        .arg(expected_value)
        .arg(ttl.as_millis() as u64)
        .invoke_async(conn)
        .await
        .map_err(|e| LeaseError::Backend(e.into()))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use redis::aio::ConnectionManager;
    use testcontainers::{ContainerAsync, GenericImage, core::WaitFor, runners::AsyncRunner};

    use super::*;

    fn test_config() -> LeaseConfig {
        LeaseConfig {
            ttl: Duration::from_secs(5),
            renewal_interval: Duration::from_millis(500),
        }
    }

    async fn setup_redis() -> (ContainerAsync<GenericImage>, RedisProvider) {
        let container = GenericImage::new("redis", "7-alpine")
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .start()
            .await
            .expect("Redis container should start");

        let port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("should get mapped port");

        let url = format!("redis://127.0.0.1:{port}");
        let (_client, conn) = crate::redis::connect(&url).await.unwrap();
        let provider = RedisProvider::new(conn, "test-instance".into(), test_config());

        (container, provider)
    }

    async fn separate_connection(container: &ContainerAsync<GenericImage>) -> ConnectionManager {
        let port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("should get mapped port");
        let url = format!("redis://127.0.0.1:{port}");
        let client = redis::Client::open(url).unwrap();
        ConnectionManager::new(client).await.unwrap()
    }

    #[tokio::test]
    async fn acquire_succeeds() {
        let (_container, provider) = setup_redis().await;
        let room_id = "acquire-succeeds".to_string();

        let lease = provider.try_acquire(&room_id).await.unwrap();
        lease.release();
    }

    #[tokio::test]
    async fn acquire_fails_when_held() {
        let (_container, provider) = setup_redis().await;
        let room_id = "acquire-fails-when-held".to_string();

        provider.try_acquire(&room_id).await.unwrap();

        let result = provider.try_acquire(&room_id).await;
        assert!(matches!(result, Err(LeaseError::HeldByOther)));
    }

    #[tokio::test]
    async fn reacquire_after_release() {
        let (_container, provider) = setup_redis().await;
        let room_id = "reacquire-after-release".to_string();

        let lease = provider.try_acquire(&room_id).await.unwrap();
        lease.release();

        // Give the background renewal task time to DEL the key
        tokio::time::sleep(Duration::from_millis(100)).await;

        provider.try_acquire(&room_id).await.unwrap();
    }

    #[tokio::test]
    async fn reacquire_after_drop() {
        let (_container, provider) = setup_redis().await;
        let room_id = "reacquire-after-drop".to_string();

        let lease = provider.try_acquire(&room_id).await.unwrap();
        drop(lease);

        // Give the background renewal task time to DEL the key
        tokio::time::sleep(Duration::from_millis(100)).await;

        provider.try_acquire(&room_id).await.unwrap();
    }

    #[tokio::test]
    async fn expired_resolves_on_key_deletion() {
        let (_container, provider) = setup_redis().await;
        let room_id = "expired-resolves".to_string();

        let mut lease = provider.try_acquire(&room_id).await.unwrap();

        let mut conn = separate_connection(&_container).await;
        let key = format!("chers:lease:{room_id}");
        let _: () = conn.del(&key).await.unwrap();

        tokio::time::timeout(Duration::from_secs(10), lease.expired())
            .await
            .expect("expired should resolve when key is deleted externally");
    }

    #[tokio::test]
    async fn renewal_keeps_key_alive() {
        let (_container, provider) = setup_redis().await;
        let room_id = "renewal-keeps-key-alive".to_string();

        let lease = provider.try_acquire(&room_id).await.unwrap();

        // Wait past one renewal interval
        tokio::time::sleep(Duration::from_secs(2)).await;

        let mut conn = separate_connection(&_container).await;
        let key = format!("chers:lease:{room_id}");
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert!(ttl > 0, "Lease TTL should still be positive after renewal");

        drop(lease);
    }
}
