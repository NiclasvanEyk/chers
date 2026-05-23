use std::sync::Arc;

use super::helpers;
use super::scenario;

// ---------------------------------------------------------------------------
// Combos that don't need Redis
// ---------------------------------------------------------------------------

/// Combo 1: All Local — LocalEventBus + LocalLease + InMemoryStorage + LocalCommandBus
#[tokio::test]
async fn all_local() {
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "all-local",
    )
    .await;
}

/// Full game test: Fool's mate (shortest checkmate) using all local components
#[tokio::test]
async fn fools_mate() {
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::fools_mate_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "fools-mate",
    )
    .await;
}

/// Combo 4: Distributed command bus over local bus
#[tokio::test]
async fn distributed_local_bus() {
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let transport = helpers::create_pubsub_transport_local().await;
    let command_bus = helpers::create_distributed_command_bus(transport);

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "distributed-local-bus",
    )
    .await;
}

// ---------------------------------------------------------------------------
// Combos that require Redis
// ---------------------------------------------------------------------------

/// Combo 2: Redis lease only
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_lease_only() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_redis(&redis).await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-lease-only",
    )
    .await;
}

/// Combo 3: Redis storage only
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_storage_only() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_redis(&redis).await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-storage-only",
    )
    .await;
}

/// Combo 5: Redis event bus only
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_bus_only() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_redis(&redis).await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-bus-only",
    )
    .await;
}

/// Combo 6: Redis bus + Redis lease
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_bus_and_lease() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_redis(&redis).await;
    let lease = helpers::create_lease_redis(&redis).await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-bus-and-lease",
    )
    .await;
}

/// Combo 7: Redis bus + Redis storage (local lease)
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_bus_and_storage() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_redis(&redis).await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_redis(&redis).await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-bus-and-storage",
    )
    .await;
}

/// Combo 8: All Redis with distributed command bus
#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_all_distributed() {
    let redis = helpers::start_redis().await;
    let event_bus = helpers::create_event_bus_redis(&redis).await;
    let lease = helpers::create_lease_redis(&redis).await;
    let storage = helpers::create_storage_redis(&redis).await;
    let transport = helpers::create_pubsub_transport_redis(&redis).await;
    let command_bus = helpers::create_distributed_command_bus(transport);

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "redis-all-distributed",
    )
    .await;
}

/// Combo 9: NATS distributed command bus (local event bus, lease, storage)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_command_bus() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let transport = helpers::create_nats_transport(&nats).await;
    let command_bus = helpers::create_distributed_command_bus(transport);

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-command-bus",
    )
    .await;
}

/// Combo 10: NATS event bus (local lease, storage, command bus)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_event_bus() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_nats(&nats).await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-event-bus",
    )
    .await;
}

/// Combo 11: NATS lease (local event bus, storage, command bus)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_lease() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_nats(&nats).await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-lease",
    )
    .await;
}

/// Combo 12: NATS event bus + NATS lease (local storage, command bus)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_bus_and_lease() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_nats(&nats).await;
    let lease = helpers::create_lease_nats(&nats).await;
    let storage = helpers::create_storage_in_memory().await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-bus-and-lease",
    )
    .await;
}

/// Combo 13: NATS storage (local event bus, lease, command bus)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_storage() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_local().await;
    let lease = helpers::create_lease_local().await;
    let storage = helpers::create_storage_nats(&nats).await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-storage",
    )
    .await;
}

/// Combo 14: All NATS (event bus + lease + storage, local command bus)
#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_all() {
    let nats = helpers::start_nats().await;
    let event_bus = helpers::create_event_bus_nats(&nats).await;
    let lease = helpers::create_lease_nats(&nats).await;
    let storage = helpers::create_storage_nats(&nats).await;
    let command_bus = helpers::create_local_command_bus();

    scenario::lifecycle_test(
        Arc::new(lease),
        Arc::new(storage),
        Arc::new(event_bus),
        Arc::new(command_bus),
        "nats-all",
    )
    .await;
}
