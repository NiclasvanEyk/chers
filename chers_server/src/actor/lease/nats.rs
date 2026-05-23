use async_nats::jetstream::kv::{CreateErrorKind, Store};
use bytes::Bytes;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

use super::{Lease, LeaseConfig, LeaseError, Provider};
use crate::room::RoomId;

const DEFAULT_KEY_PREFIX: &str = "chers.lease.";

pub struct NatsProvider {
    kv: Store,
    config: LeaseConfig,
    instance_id: String,
    key_prefix: String,
}

impl NatsProvider {
    pub fn new(kv: Store, config: LeaseConfig, instance_id: String) -> Self {
        Self::with_prefix(kv, config, instance_id, DEFAULT_KEY_PREFIX.into())
    }

    pub fn with_prefix(
        kv: Store,
        config: LeaseConfig,
        instance_id: String,
        key_prefix: String,
    ) -> Self {
        Self {
            kv,
            config,
            instance_id,
            key_prefix,
        }
    }
}

impl Provider for NatsProvider {
    async fn try_acquire(&self, room_id: &RoomId) -> Result<Lease, LeaseError> {
        let key = format!("{}{room_id}", self.key_prefix);

        let result = self
            .kv
            .create(&key, Bytes::from(self.instance_id.as_bytes().to_vec()))
            .await;

        match result {
            Ok(_revision) => {
                info!(%room_id, "Acquired lease");

                let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
                let (expired_tx, expired_rx) = oneshot::channel::<()>();

                let kv = self.kv.clone();
                let renew_key = key.clone();
                let renew_instance_id = self.instance_id.clone();
                let renew_interval = self.config.renewal_interval;

                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(renew_interval);
                    interval.tick().await;

                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                match renew_lease(&kv, &renew_key, &renew_instance_id).await {
                                    Ok(true) => {}
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
                                let _ = kv.delete(&renew_key).await;
                                return;
                            }
                        }
                    }
                });

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
            Err(e) if e.kind() == CreateErrorKind::AlreadyExists => Err(LeaseError::HeldByOther),
            Err(e) => Err(LeaseError::Backend(e.into())),
        }
    }
}

async fn renew_lease(kv: &Store, key: &str, expected_value: &str) -> Result<bool, LeaseError> {
    let entry = kv
        .entry(key)
        .await
        .map_err(|e| LeaseError::Backend(e.into()))?;

    match entry {
        Some(entry) if entry.value.as_ref() == expected_value.as_bytes() => {
            kv.update(key, entry.value.clone(), entry.revision)
                .await
                .map_err(|e| LeaseError::Backend(e.into()))?;
            Ok(true)
        }
        Some(_) => Ok(false),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use testcontainers::core::IntoContainerPort;
    use testcontainers::{
        ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner,
    };
    fn test_config() -> LeaseConfig {
        LeaseConfig {
            ttl: Duration::from_secs(10),
            renewal_interval: Duration::from_millis(500),
        }
    }

    struct TestContext {
        _container: ContainerAsync<GenericImage>,
        provider: NatsProvider,
        kv: async_nats::jetstream::kv::Store,
    }

    async fn setup(name: &str) -> TestContext {
        let container = GenericImage::new("nats", "2.10-alpine")
            .with_exposed_port(4222.tcp())
            .with_wait_for(WaitFor::seconds(1))
            .with_cmd(["--jetstream"])
            .start()
            .await
            .expect("NATS container should start — is Docker running?");

        let port = container
            .get_host_port_ipv4(4222)
            .await
            .expect("should get mapped port");

        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                if tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
                    .await
                    .is_ok()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("NATS port should become reachable within 30s");

        let config = test_config();
        let url = format!("nats://127.0.0.1:{port}");
        let client = async_nats::connect(&url).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);
        let bucket = format!("chers_lease_{name}");

        let kv = jetstream
            .create_key_value(async_nats::jetstream::kv::Config {
                bucket,
                max_age: config.ttl,
                history: 1,
                ..Default::default()
            })
            .await
            .expect("should create KV bucket");

        let prefix = format!("test.{name}.");
        let provider =
            NatsProvider::with_prefix(kv.clone(), config, "test-instance".into(), prefix);

        TestContext {
            _container: container,
            provider,
            kv,
        }
    }

    #[tokio::test]
    async fn acquire_succeeds() {
        let ctx = setup("acquire_succeeds").await;
        let lease = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        lease.release();
    }

    #[tokio::test]
    async fn acquire_fails_when_held() {
        let ctx = setup("acquire_fails_when_held").await;
        let _first = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        let result = ctx.provider.try_acquire(&"r1".into()).await;
        assert!(matches!(result, Err(LeaseError::HeldByOther)));
    }

    #[tokio::test]
    async fn reacquire_after_release() {
        let ctx = setup("reacquire_after_release").await;
        let lease = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        lease.release();
        tokio::time::sleep(Duration::from_millis(200)).await;
        ctx.provider.try_acquire(&"r1".into()).await.unwrap();
    }

    #[tokio::test]
    async fn reacquire_after_drop() {
        let ctx = setup("reacquire_after_drop").await;
        let lease = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        drop(lease);
        tokio::time::sleep(Duration::from_millis(200)).await;
        ctx.provider.try_acquire(&"r1".into()).await.unwrap();
    }

    #[tokio::test]
    async fn expired_resolves_on_key_deletion() {
        let ctx = setup("expired_resolves").await;
        let mut lease = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        let key = format!("test.expired_resolves.r1");
        ctx.kv.delete(&key).await.unwrap();
        tokio::time::timeout(Duration::from_secs(10), lease.expired())
            .await
            .expect("expired should resolve when key is deleted externally");
    }

    #[tokio::test]
    async fn renewal_keeps_key_alive() {
        let ctx = setup("renewal_keeps_key_alive").await;
        let lease = ctx.provider.try_acquire(&"r1".into()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
        let key = format!("test.renewal_keeps_key_alive.r1");
        let entry = ctx
            .kv
            .entry(&key)
            .await
            .expect("should read entry")
            .expect("entry should exist after renewal");
        assert_eq!(
            entry.value.as_ref(),
            b"test-instance",
            "Lease value should still be ours after renewal"
        );
        drop(lease);
    }
}
