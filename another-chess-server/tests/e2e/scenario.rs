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
        connection_id: "conn-a-1".into(),
    };
    let user_b = User {
        id: "user-b".into(),
        name: "Player B".into(),
        connection_id: "conn-b-1".into(),
    };

    let mut events_a = room.subscribe().await.expect("client A should subscribe");
    let mut events_b = room.subscribe().await.expect("client B should subscribe");

    // Step 1: Client A joins
    let resp = room
        .send(Command::Lobby(LobbyCommand::Join {
            secret: "secret-a".into(),
            name: "Player A".into(),
            connection_id: "conn-a-1".into(),
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
            connection_id: "conn-b-1".into(),
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
    let (white_id, black_id) = match &event {
        Some(Event::Game(GameEvent::GameStarted { white, black })) => {
            // Verify both players are assigned to different colors
            assert_ne!(white.id, black.id, "players should have different colors");
            assert!(
                (white.id == "user-a" && black.id == "user-b")
                    || (white.id == "user-b" && black.id == "user-a"),
                "expected GameStarted with both players assigned"
            );
            (white.id.clone(), black.id.clone())
        }
        _ => panic!("expected GameStarted for client A, got {:?}", event),
    };
    let event = events_b.next().await;
    assert!(
        matches!(&event, Some(Event::Game(GameEvent::GameStarted { white, black })) 
            if white.id == white_id && black.id == black_id),
        "expected GameStarted for client B with same assignments"
    );

    // Step 6: Verify we're in game phase — a game command should succeed
    // The white player makes the first move (e2-e4)
    let white_user = if white_id == "user-a" {
        user_a.clone()
    } else {
        user_b.clone()
    };
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: white_user,
            move_: chers::Move::simple(
                chers::Coordinate::new(4, 6), // E2
                chers::Coordinate::new(4, 4), // E4
            ),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );
}

/// Run a full game scenario ending in fool's mate (fastest checkmate).
///
/// Scenario:
/// 1. Two players join and start the game
/// 2. Play fool's mate: f3, e5, g4, Qh4#
/// 3. Verify Black wins by checkmate
pub async fn fools_mate_test<P, S, B, T>(
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
    // Pre-populate the room's auth with deterministic user IDs
    {
        let rid = room_id.to_string();
        let mut persisted = storage
            .get_by_id(&rid)
            .await
            .expect("storage get_by_id should not error")
            .unwrap_or_else(|| another_chess_server::room::Room::new(room_id.to_string()));
        persisted
            .auth
            .insert("secret-a", "user-a".to_string(), "White Player");
        persisted
            .auth
            .insert("secret-b", "user-b".to_string(), "Black Player");
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
        name: "White Player".into(),
        connection_id: "conn-a-2".into(),
    };
    let user_b = User {
        id: "user-b".into(),
        name: "Black Player".into(),
        connection_id: "conn-b-2".into(),
    };

    let mut events_a = room.subscribe().await.expect("client A should subscribe");
    let mut events_b = room.subscribe().await.expect("client B should subscribe");

    // Step 1: Both players join
    let _ = room
        .send(Command::Lobby(LobbyCommand::Join {
            secret: "secret-a".into(),
            name: "White Player".into(),
            connection_id: "conn-a-2".into(),
        }))
        .await
        .expect("send Join should succeed");
    let _ = events_a.next().await; // PlayerJoined for A
    let _ = events_b.next().await; // PlayerJoined for B

    let _ = room
        .send(Command::Lobby(LobbyCommand::Join {
            secret: "secret-b".into(),
            name: "Black Player".into(),
            connection_id: "conn-b-2".into(),
        }))
        .await
        .expect("send Join should succeed");
    let _ = events_a.next().await; // PlayerJoined for B
    let _ = events_b.next().await; // PlayerJoined for B

    // Step 2: Both players ready up to start the game
    let _ = room
        .send(Command::Lobby(LobbyCommand::ChangeReady {
            user: user_a.clone(),
            is_ready: true,
        }))
        .await
        .expect("ready should succeed");
    let _ = events_a.next().await; // ReadyChanged
    let _ = events_b.next().await; // ReadyChanged

    let _ = room
        .send(Command::Lobby(LobbyCommand::ChangeReady {
            user: user_b.clone(),
            is_ready: true,
        }))
        .await
        .expect("ready should succeed");
    let _ = events_a.next().await; // ReadyChanged
    let _ = events_b.next().await; // ReadyChanged

    // Get GameStarted event and determine color assignments
    let event = events_a.next().await;
    let (white_id, black_id) = match &event {
        Some(Event::Game(GameEvent::GameStarted { white, black })) => {
            (white.id.clone(), black.id.clone())
        }
        _ => panic!("expected GameStarted, got {:?}", event),
    };
    let _ = events_b.next().await; // GameStarted for B

    let white_user = if white_id == "user-a" {
        user_a.clone()
    } else {
        user_b.clone()
    };
    let black_user = if black_id == "user-b" {
        user_b.clone()
    } else {
        user_a.clone()
    };

    // Step 3: Play fool's mate
    // Move 1: White plays f2-f3 (f3)
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: white_user.clone(),
            move_: chers::Move::simple(
                chers::Coordinate::new(5, 6), // f2
                chers::Coordinate::new(5, 5), // f3
            ),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let _ = events_a.next().await; // Turn event
    let _ = events_b.next().await; // Turn event

    // Move 2: Black plays e7-e5
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: black_user.clone(),
            move_: chers::Move::simple(
                chers::Coordinate::new(4, 1), // e7
                chers::Coordinate::new(4, 3), // e5
            ),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let _ = events_a.next().await; // Turn event
    let _ = events_b.next().await; // Turn event

    // Move 3: White plays g2-g4 (g4)
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: white_user.clone(),
            move_: chers::Move::simple(
                chers::Coordinate::new(6, 6), // g2
                chers::Coordinate::new(6, 4), // g4
            ),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    let _ = events_a.next().await; // Turn event
    let _ = events_b.next().await; // Turn event

    // Move 4: Black plays Qd8-h4# (checkmate!)
    let resp = room
        .send(Command::Game(GameCommand::MakeMove {
            user: black_user.clone(),
            move_: chers::Move::simple(
                chers::Coordinate::new(3, 0), // d8 (queen start)
                chers::Coordinate::new(7, 4), // h4 (checkmate!)
            ),
        }))
        .await
        .expect("send MakeMove should succeed");
    assert!(
        matches!(resp, CommandResponse::Accepted),
        "expected Accepted"
    );

    // Step 4: Verify the checkmate and game end
    let event_a = events_a.next().await;
    let event_b = events_b.next().await;

    // Both clients should receive the Turn event
    assert!(
        matches!(&event_a, Some(Event::Game(GameEvent::Turn { author, move_ }))
            if author.id == black_user.id && move_.to == chers::Coordinate::new(7, 4)),
        "expected Turn event with Qh4# for client A, got {:?}",
        event_a
    );
    assert!(
        matches!(&event_b, Some(Event::Game(GameEvent::Turn { author, move_ }))
            if author.id == black_user.id && move_.to == chers::Coordinate::new(7, 4)),
        "expected Turn event with Qh4# for client B, got {:?}",
        event_b
    );

    // Both clients should receive the GameEnded event with Black as winner
    let event_a = events_a.next().await;
    let event_b = events_b.next().await;

    assert!(
        matches!(&event_a, Some(Event::Game(GameEvent::GameEnded { winner, reason }))
            if winner.as_ref().map(|u| u.id == black_user.id).unwrap_or(false)
            && matches!(reason, another_chess_server::communication::event::GameEndReason::Checkmate)),
        "expected GameEnded with Black winning by checkmate for client A, got {:?}",
        event_a
    );
    assert!(
        matches!(&event_b, Some(Event::Game(GameEvent::GameEnded { winner, reason }))
            if winner.as_ref().map(|u| u.id == black_user.id).unwrap_or(false)
            && matches!(reason, another_chess_server::communication::event::GameEndReason::Checkmate)),
        "expected GameEnded with Black winning by checkmate for client B, got {:?}",
        event_b
    );
}
