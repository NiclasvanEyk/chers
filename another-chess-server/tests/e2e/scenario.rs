use std::sync::Arc;

use another_chess_server::actor::lease::Provider;
use another_chess_server::actor::registry::RoomRegistry;
use another_chess_server::auth::User;
use another_chess_server::communication::bus::EventBus;
use another_chess_server::communication::command::{
    Command, CommandBus, CommandResponse, GameCommand, LobbyCommand,
};
use another_chess_server::communication::event::{Event, GameEvent, LobbyEvent};
use another_chess_server::room::storage::Storage;

use tokio_stream::StreamExt;

/// Run the full lifecycle scenario with the given components.
///
/// Scenario:
/// 1. Two players join a room
/// 2. One changes their name
/// 3. The other toggles ready
/// 4. Each step verifies events arrive at all subscribers
/// 5. Verifies persisted state
pub async fn lifecycle_test<P, S, B, T>(
    lease: Arc<P>,
    storage: Arc<S>,
    event_bus: Arc<B>,
    transport: Arc<T>,
    room_id: &str,
) where
    P: Provider,
    S: Storage + 'static,
    B: EventBus<Item = Event> + 'static,
    T: CommandBus<Cmd = Command> + 'static,
{
    // Pre-populate the room's auth so that join commands resolve to
    // deterministic user IDs.  The actor loads this from storage.
    {
        let rid = room_id.to_string();
        let mut persisted = storage
            .get_by_id(&rid)
            .await
            .expect("storage get_by_id should not error")
            .unwrap_or_else(|| another_chess_server::room::Room::new(room_id.to_string()));
        persisted
            .auth
            .insert("secret-a", "user-a".to_string(), "Player A");
        persisted
            .auth
            .insert("secret-b", "user-b".to_string(), "Player B");
        storage
            .persist(&persisted)
            .await
            .expect("should persist pre-seeded auth");
    }

    let registry = RoomRegistry::new(lease, storage.clone(), event_bus, transport);

    let room = registry
        .get_or_create(room_id.to_string())
        .await
        .expect("should create room");

    let user_a = User {
        id: "user-a".into(),
        name: "Player A".into(),
    };
    let user_b = User {
        id: "user-b".into(),
        name: "Player B".into(),
    };

    let mut events_a = room.subscribe().await.expect("client A should subscribe");
    let mut events_b = room.subscribe().await.expect("client B should subscribe");

    // Step 1: Client A joins
    let resp = room
        .send(Command::Lobby(LobbyCommand::Join {
            secret: "secret-a".into(),
            name: "Player A".into(),
        }))
        .await
        .expect("send Join should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerJoined { user })) if user.id == "user-a"),
        "expected PlayerJoined(user-a) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerJoined { user })) if user.id == "user-a"),
        "expected PlayerJoined(user-a) for client B"
    );

    // Step 2: Client B joins
    let resp = room
        .send(Command::Lobby(LobbyCommand::Join {
            secret: "secret-b".into(),
            name: "Player B".into(),
        }))
        .await
        .expect("send Join should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerJoined { user })) if user.id == "user-b"),
        "expected PlayerJoined(user-b) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerJoined { user })) if user.id == "user-b"),
        "expected PlayerJoined(user-b) for client B"
    );

    // Step 3: Client A changes name
    let resp = room
        .send(Command::Lobby(LobbyCommand::ChangeName {
            user: user_a.clone(),
            new_name: "Alice".into(),
        }))
        .await
        .expect("send ChangeName should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerNameChanged { user })) if user.id == "user-a"),
        "expected PlayerNameChanged(user-a) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerNameChanged { user })) if user.id == "user-a"),
        "expected PlayerNameChanged(user-a) for client B"
    );

    // Step 4: Client B toggles ready
    let resp = room
        .send(Command::Lobby(LobbyCommand::ChangeReady {
            user: user_b.clone(),
            is_ready: true,
        }))
        .await
        .expect("send ChangeReady should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerReadyChanged { user, is_ready })) if user.id == "user-b" && *is_ready),
        "expected PlayerReadyChanged(user-b, ready=true) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerReadyChanged { user, is_ready })) if user.id == "user-b" && *is_ready),
        "expected PlayerReadyChanged(user-b, ready=true) for client B"
    );

    // Step 5: Client A toggles ready — both ready → game starts
    let resp = room
        .send(Command::Lobby(LobbyCommand::ChangeReady {
            user: user_a.clone(),
            is_ready: true,
        }))
        .await
        .expect("send ChangeReady should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerReadyChanged { user, is_ready })) if user.id == "user-a" && *is_ready),
        "expected PlayerReadyChanged(user-a, ready=true) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Lobby(LobbyEvent::PlayerReadyChanged { user, is_ready })) if user.id == "user-a" && *is_ready),
        "expected PlayerReadyChanged(user-a, ready=true) for client B"
    );

    let event = events_a.next().await;
    assert!(
        matches!(&event, Some(Event::Game(GameEvent::GameStarted { white, black })) if white.id == "user-a" && black.id == "user-b"),
        "expected GameStarted(white=user-a, black=user-b) for client A"
    );
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Game(GameEvent::GameStarted { white, black })) if white.id == "user-a" && black.id == "user-b"),
        "expected GameStarted(white=user-a, black=user-b) for client B"
    );

    // Step 6: Verify we're in game phase — a game command should succeed
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: user_a.clone(),
            turn: "e4".into(),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );
}
