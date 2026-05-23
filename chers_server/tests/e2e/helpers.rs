use std::sync::Arc;
use std::time::Duration;

use chers_server::actor::lease::{AnyLease, LeaseConfig, LocalLease};
use chers_server::communication::bus::{AnyEventBus, LocalEventBus};
use chers_server::communication::command::{DistributedCommandBus, LocalCommandBus};
use chers_server::communication::event::Event;
use chers_server::communication::transport::CommandTransport;
use chers_server::communication::transport::pubsub::PubSubCommandTransport;
use chers_server::room::storage::{AnyStorage, InMemoryStorage};

#[cfg(feature = "redis")]
use chers_server::actor::lease::RedisProvider;
#[cfg(feature = "redis")]
use chers_server::communication::bus::RedisEventBus;
#[cfg(feature = "redis")]
use chers_server::room::storage::RedisStorage;

#[cfg(feature = "nats")]
use chers_server::actor::lease::NatsProvider;
#[cfg(feature = "nats")]
use chers_server::communication::bus::NatsEventBus;
#[cfg(feature = "nats")]
use chers_server::room::storage::NatsStorage;

use testcontainers::core::IntoContainerPort;
use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};

#[cfg(feature = "redis")]
pub struct RedisInstance {
    _container: ContainerAsync<GenericImage>,
    pub url: String,
}

#[cfg(feature = "redis")]
pub async fn start_redis() -> RedisInstance {
    let container = GenericImage::new("redis", "8-alpine")
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await
        .expect("Redis container should start — is Docker running?");

    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("should get mapped port");

    RedisInstance {
        _container: container,
        url: format!("redis://127.0.0.1:{port}"),
    }
}

#[cfg(feature = "nats")]
pub struct NatsInstance {
    _container: ContainerAsync<GenericImage>,
    pub url: String,
}

#[cfg(feature = "nats")]
pub async fn start_nats() -> NatsInstance {
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

    NatsInstance {
        _container: container,
        url: format!("nats://127.0.0.1:{port}"),
    }
}

pub async fn create_event_bus_local() -> AnyEventBus<Event> {
    AnyEventBus::Local(LocalEventBus::new())
}

pub async fn create_pubsub_transport_local() -> PubSubCommandTransport<AnyEventBus<Vec<u8>>> {
    PubSubCommandTransport::new(Arc::new(AnyEventBus::Local(LocalEventBus::new())))
}

pub async fn create_lease_local() -> AnyLease {
    AnyLease::Local(LocalLease::new())
}

pub async fn create_storage_in_memory() -> AnyStorage {
    AnyStorage::InMemory(InMemoryStorage::new())
}

#[cfg(feature = "redis")]
pub async fn create_event_bus_redis(redis: &RedisInstance) -> AnyEventBus<Event> {
    let (client, conn) = chers_server::redis::connect(&redis.url).await.unwrap();
    AnyEventBus::Redis(RedisEventBus::new(client, conn))
}

#[cfg(feature = "redis")]
pub async fn create_pubsub_transport_redis(
    redis: &RedisInstance,
) -> PubSubCommandTransport<AnyEventBus<Vec<u8>>> {
    let (client, conn) = chers_server::redis::connect(&redis.url).await.unwrap();
    PubSubCommandTransport::new(Arc::new(AnyEventBus::Redis(RedisEventBus::new(
        client, conn,
    ))))
}

#[cfg(feature = "redis")]
pub async fn create_lease_redis(redis: &RedisInstance) -> AnyLease {
    let (_client, conn) = chers_server::redis::connect(&redis.url).await.unwrap();
    let config = LeaseConfig {
        ttl: Duration::from_secs(5),
        renewal_interval: Duration::from_millis(500),
    };
    AnyLease::Redis(RedisProvider::new(
        conn,
        format!("test-{}", std::process::id()),
        config,
    ))
}

#[cfg(feature = "redis")]
pub async fn create_storage_redis(redis: &RedisInstance) -> AnyStorage {
    let (_client, conn) = chers_server::redis::connect(&redis.url).await.unwrap();
    AnyStorage::Redis(RedisStorage::new(conn))
}

#[cfg(feature = "nats")]
pub async fn create_nats_transport(
    nats: &NatsInstance,
) -> chers_server::communication::transport::nats::NatsCommandTransport {
    let client = async_nats::connect(&nats.url)
        .await
        .expect("should connect to NATS");
    chers_server::communication::transport::nats::NatsCommandTransport::new(client)
}

#[cfg(feature = "nats")]
pub async fn create_event_bus_nats(nats: &NatsInstance) -> AnyEventBus<Event> {
    let client = async_nats::connect(&nats.url)
        .await
        .expect("should connect to NATS");
    AnyEventBus::Nats(NatsEventBus::new(client))
}

#[cfg(feature = "nats")]
pub async fn create_lease_nats(nats: &NatsInstance) -> AnyLease {
    let client = async_nats::connect(&nats.url)
        .await
        .expect("should connect to NATS");
    let jetstream = async_nats::jetstream::new(client);
    let config = LeaseConfig {
        ttl: Duration::from_secs(10),
        renewal_interval: Duration::from_millis(500),
    };
    let kv = jetstream
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "chers_lease_test".to_string(),
            max_age: config.ttl,
            history: 1,
            ..Default::default()
        })
        .await
        .expect("should create KV bucket");
    AnyLease::Nats(NatsProvider::new(
        kv,
        config,
        format!("test-{}", std::process::id()),
    ))
}

#[cfg(feature = "nats")]
pub async fn create_storage_nats(nats: &NatsInstance) -> AnyStorage {
    let client = async_nats::connect(&nats.url)
        .await
        .expect("should connect to NATS");
    let jetstream = async_nats::jetstream::new(client);
    let kv = jetstream
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "chers_room_test".to_string(),
            history: 1,
            ..Default::default()
        })
        .await
        .expect("should create KV bucket");
    AnyStorage::Nats(NatsStorage::new(kv))
}

pub fn create_local_command_bus() -> LocalCommandBus {
    LocalCommandBus::new()
}

pub fn create_distributed_command_bus<T: CommandTransport>(
    transport: T,
) -> DistributedCommandBus<T> {
    DistributedCommandBus::new(Arc::new(transport))
}
