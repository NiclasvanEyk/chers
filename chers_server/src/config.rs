use std::sync::Arc;
use std::{env, error::Error};

#[cfg(any(feature = "redis", feature = "nats"))]
use crate::actor::lease::LeaseConfig;
#[cfg(feature = "nats")]
use crate::actor::lease::NatsProvider;
use crate::actor::lease::{AnyLease, LocalLease};
use crate::actor::registry::RoomRegistry;
use crate::communication::{
    bus::{AnyEventBus, EventBusDriver, LocalEventBus},
    command::{AnyCommandBus, DistributedCommandBus, LocalCommandBus},
    event::Event,
    transport::pubsub::PubSubCommandTransport,
};
use crate::room::storage::{AnyStorage, InMemoryStorage};

#[cfg(feature = "redis")]
use crate::actor::lease::RedisProvider;
#[cfg(feature = "nats")]
use crate::communication::bus::NatsEventBus;
#[cfg(feature = "redis")]
use crate::communication::bus::RedisEventBus;
#[cfg(feature = "redis")]
use crate::redis;
#[cfg(feature = "nats")]
use crate::room::storage::NatsStorage;
#[cfg(feature = "redis")]
use crate::room::storage::RedisStorage;

pub async fn event_bus() -> Result<AnyEventBus<Event>, Box<dyn Error>> {
    Ok(match driver("EVENT_BUS", EventBusDriver::Local)? {
        EventBusDriver::Local => AnyEventBus::Local(LocalEventBus::<Event>::new()),
        #[cfg(feature = "redis")]
        EventBusDriver::Redis => {
            let (client, conn) = redis::from_env("CHERS_EVENT_BUS").await?;
            AnyEventBus::Redis(RedisEventBus::new(client, conn))
        }
        #[cfg(feature = "nats")]
        EventBusDriver::Nats => {
            let client = connect_nats().await?;
            AnyEventBus::Nats(NatsEventBus::new(client))
        }
    })
}

pub async fn lease_provider() -> Result<AnyLease, Box<dyn Error>> {
    Ok(match driver("LEASE", LeaseDriver::Local)? {
        LeaseDriver::Local => AnyLease::Local(LocalLease::new()),
        #[cfg(feature = "redis")]
        LeaseDriver::Redis => {
            let (_client, conn) = redis::from_env("CHERS_LEASE").await?;
            let config = LeaseConfig::from_env();
            AnyLease::Redis(RedisProvider::new(
                conn,
                format!("server-{}", std::process::id()),
                config,
            ))
        }
        #[cfg(feature = "nats")]
        LeaseDriver::Nats => {
            let client = connect_nats().await?;
            let jetstream = async_nats::jetstream::new(client);
            let config = LeaseConfig::from_env();
            let kv = get_or_create_lease_bucket(&jetstream, config.ttl).await?;
            AnyLease::Nats(NatsProvider::new(
                kv,
                config,
                format!("server-{}", std::process::id()),
            ))
        }
    })
}

pub async fn storage() -> Result<AnyStorage, Box<dyn Error>> {
    Ok(match driver("STORAGE", StorageDriver::InMemory)? {
        StorageDriver::InMemory => AnyStorage::InMemory(InMemoryStorage::new()),
        #[cfg(feature = "redis")]
        StorageDriver::Redis => {
            let (_client, conn) = redis::from_env("CHERS_STORAGE").await?;
            AnyStorage::Redis(RedisStorage::new(conn))
        }
        #[cfg(feature = "nats")]
        StorageDriver::Nats => {
            let client = connect_nats().await?;
            let jetstream = async_nats::jetstream::new(client);
            let kv = get_or_create_storage_bucket(&jetstream).await?;
            AnyStorage::Nats(NatsStorage::new(kv))
        }
    })
}

pub async fn command_bus() -> Result<AnyCommandBus, Box<dyn Error>> {
    Ok(match driver("COMMAND_BUS", CommandBusDriver::Local)? {
        CommandBusDriver::Local => AnyCommandBus::Local(LocalCommandBus::new()),
        CommandBusDriver::Distributed => {
            let transport = build_command_transport().await?;
            AnyCommandBus::Distributed(DistributedCommandBus::new(Arc::new(transport)))
        }
        #[cfg(feature = "nats")]
        CommandBusDriver::Nats => {
            let transport = build_nats_transport().await?;
            AnyCommandBus::Nats(DistributedCommandBus::new(Arc::new(transport)))
        }
    })
}

/// Connect to the NATS server using `CHERS_NATS_URL` (or `NATS_URL`),
/// optionally authenticating with a `.creds` file from `CHERS_NATS_CREDS` (or `NATS_CREDS`).
#[cfg(feature = "nats")]
async fn connect_nats() -> Result<async_nats::Client, Box<dyn Error>> {
    let url = resolve_nats_url()?;
    match resolve_nats_creds() {
        Some(path) => {
            let nc = async_nats::ConnectOptions::with_credentials_file(path)
                .await?
                .connect(&url)
                .await?;
            Ok(nc)
        }
        None => {
            let nc = async_nats::connect(&url).await?;
            Ok(nc)
        }
    }
}

/// Resolve a NATS credentials file from `CHERS_NATS_CREDS` env var,
/// falling back to `NATS_CREDS`.
#[cfg(feature = "nats")]
fn resolve_nats_creds() -> Option<std::path::PathBuf> {
    std::env::var_os("CHERS_NATS_CREDS")
        .or_else(|| std::env::var_os("NATS_CREDS"))
        .map(std::path::PathBuf::from)
}

/// Create a [`NatsCommandTransport`] from the NATS connection.
#[cfg(feature = "nats")]
async fn build_nats_transport()
-> Result<crate::communication::transport::nats::NatsCommandTransport, Box<dyn Error>> {
    let client = connect_nats().await?;
    Ok(crate::communication::transport::nats::NatsCommandTransport::new(client))
}

/// Resolve a NATS URL from `CHERS_NATS_URL` env var, falling back to `NATS_URL`.
#[cfg(feature = "nats")]
fn resolve_nats_url() -> Result<String, Box<dyn Error>> {
    Ok(std::env::var_os("CHERS_NATS_URL")
        .or_else(|| std::env::var_os("NATS_URL"))
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "nats://localhost:4222".to_string()))
}

/// Create a [`PubSubCommandTransport`] using the **event bus** driver
/// setting.  The command transport is built on top of the same bus backend
/// so that a single Redis (or future NATS) instance serves both event
/// distribution and command request-reply.
async fn build_command_transport()
-> Result<PubSubCommandTransport<AnyEventBus<Vec<u8>>>, Box<dyn Error>> {
    let bus: AnyEventBus<Vec<u8>> = match driver("EVENT_BUS", EventBusDriver::Local)? {
        EventBusDriver::Local => AnyEventBus::Local(LocalEventBus::<Vec<u8>>::new()),
        #[cfg(feature = "redis")]
        EventBusDriver::Redis => {
            let (client, conn) = redis::from_env("CHERS_EVENT_BUS").await?;
            AnyEventBus::Redis(RedisEventBus::new(client, conn))
        }
        #[cfg(feature = "nats")]
        EventBusDriver::Nats => {
            let client = connect_nats().await?;
            AnyEventBus::Nats(NatsEventBus::new(client))
        }
    };
    Ok(PubSubCommandTransport::new(Arc::new(bus)))
}

fn driver<T>(name: &'static str, default: T) -> Result<T, serde_plain::Error>
where
    T: serde::de::DeserializeOwned,
{
    env::var_os(format!("CHERS_{name}_DRIVER"))
        .map(|s| serde_plain::from_str::<T>(&s.to_string_lossy()))
        .unwrap_or(Ok(default))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum CommandBusDriver {
    Local,
    Distributed,
    #[cfg(feature = "nats")]
    Nats,
}

/// Get or create the NATS JetStream KV bucket used for leases.
#[cfg(feature = "nats")]
async fn get_or_create_lease_bucket(
    jetstream: &async_nats::jetstream::Context,
    ttl: std::time::Duration,
) -> Result<async_nats::jetstream::kv::Store, Box<dyn Error>> {
    if let Ok(store) = jetstream.get_key_value("chers_lease").await {
        return Ok(store);
    }

    let max_bytes = env::var("CHERS_NATS_LEASE_BUCKET_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1_048_576); // 1 MB default

    let store = jetstream
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "chers_lease".to_string(),
            max_age: ttl,
            max_bytes,
            history: 1,
            ..Default::default()
        })
        .await?;

    Ok(store)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum LeaseDriver {
    Local,
    #[cfg(feature = "redis")]
    Redis,
    #[cfg(feature = "nats")]
    Nats,
}

/// Get or create the NATS JetStream KV bucket used for room storage.
#[cfg(feature = "nats")]
async fn get_or_create_storage_bucket(
    jetstream: &async_nats::jetstream::Context,
) -> Result<async_nats::jetstream::kv::Store, Box<dyn Error>> {
    if let Ok(store) = jetstream.get_key_value("chers_room").await {
        return Ok(store);
    }

    let max_bytes = env::var("CHERS_NATS_STORAGE_BUCKET_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10_485_760); // 10 MB default

    let store = jetstream
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "chers_room".to_string(),
            max_bytes,
            history: 1,
            ..Default::default()
        })
        .await?;

    Ok(store)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum StorageDriver {
    InMemory,
    #[cfg(feature = "redis")]
    Redis,
    #[cfg(feature = "nats")]
    Nats,
}

pub async fn room_registry()
-> Result<RoomRegistry<AnyLease, AnyStorage, AnyEventBus<Event>, AnyCommandBus>, Box<dyn Error>> {
    Ok(RoomRegistry::new(
        Arc::new(lease_provider().await?),
        Arc::new(storage().await?),
        Arc::new(event_bus().await?),
        Arc::new(command_bus().await?),
    ))
}

pub fn server_address() -> String {
    std::env::var("PORT")
        .map(|p| format!("0.0.0.0:{p}"))
        .or_else(|_| std::env::var("CHERS_ADDR"))
        .unwrap_or_else(|_| "0.0.0.0:8000".into())
}
