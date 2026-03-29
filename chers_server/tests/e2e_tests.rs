mod common;

use common::{TestClient, TestServer};

#[tokio::test]
async fn test_server_starts_and_responds_to_health() {
    let server = TestServer::start().await;

    // Give server a bit more time to fully start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test health endpoint
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            println!("Health check response: status={}, body={}", status, body);
            assert!(
                status.is_success(),
                "Health endpoint returned error status: {}",
                status
            );
            assert!(
                body.contains("Perfectly fine"),
                "Health check message unexpected: {}",
                body
            );
        }
        Err(e) => {
            panic!("Failed to connect to health endpoint: {:?}", e);
        }
    }

    server.stop();
}

#[tokio::test]
async fn test_create_match() {
    let server = TestServer::start().await;

    let match_id = server.create_match().await;

    // Verify it's a valid UUID
    assert_eq!(match_id.len(), 36, "Match ID should be a UUID (36 chars)");
    assert!(match_id.contains('-'), "Match ID should contain hyphens");

    server.stop();
}

#[tokio::test]
async fn test_two_players_connect_and_start_game() {
    let server = TestServer::start().await;

    // Create match
    let match_id = server.create_match().await;

    // Connect both players
    let mut player1 = TestClient::connect(&server, &match_id).await;
    let mut player2 = TestClient::connect(&server, &match_id).await;

    // Authenticate
    player1.authenticate("alice", "Alice").await;
    player2.authenticate("bob", "Bob").await;

    // Both players receive Authenticated first, then GameStarted
    // (Authenticated is sent immediately, GameStarted when both are connected)
    use chers_server_api::{PublicEvent, ServerMessage};

    let auth1 = player1.expect_message(5).await;
    println!("Player 1 auth: {:?}", auth1);
    let auth2 = player2.expect_message(5).await;
    println!("Player 2 auth: {:?}", auth2);

    assert!(
        matches!(
            auth1,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::Authenticated { .. }
            ))
        ),
        "Player 1 should receive Authenticated, got: {:?}",
        auth1
    );

    assert!(
        matches!(
            auth2,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::Authenticated { .. }
            ))
        ),
        "Player 2 should receive Authenticated, got: {:?}",
        auth2
    );

    // Then both receive GameStarted
    let msg1 = player1.expect_message(5).await;
    println!("Player 1 game started: {:?}", msg1);
    let msg2 = player2.expect_message(5).await;
    println!("Player 2 game started: {:?}", msg2);

    assert!(
        matches!(
            msg1,
            Some(ServerMessage::Public(PublicEvent::GameStarted { .. }))
        ),
        "Player 1 should receive GameStarted, got: {:?}",
        msg1
    );

    assert!(
        matches!(
            msg2,
            Some(ServerMessage::Public(PublicEvent::GameStarted { .. }))
        ),
        "Player 2 should receive GameStarted, got: {:?}",
        msg2
    );

    // Cleanup
    player1.close().await;
    player2.close().await;
    server.stop();
}

#[tokio::test]
async fn test_turn_enforcement() {
    let server = TestServer::start().await;

    let match_id = server.create_match().await;

    let mut player1 = TestClient::connect(&server, &match_id).await;
    let mut player2 = TestClient::connect(&server, &match_id).await;

    // Authenticate both players
    player1.authenticate("player1", "Player1").await;
    player2.authenticate("player2", "Player2").await;

    // Both players should receive Authenticated then GameStarted
    let _ = player1.expect_message(5).await; // Authenticated
    let game_started_1 = player1.expect_message(5).await; // GameStarted
    let _ = player2.expect_message(5).await; // Authenticated
    let game_started_2 = player2.expect_message(5).await; // GameStarted

    use chers_server_api::{Color, PublicEvent, ServerMessage};

    // Extract which player is playing which color from GameStarted
    let (player1_color, player2_color) = match (&game_started_1, &game_started_2) {
        (
            Some(ServerMessage::Public(PublicEvent::GameStarted {
                white_player,
                black_player,
                ..
            })),
            _,
        ) => {
            // Determine which client is which color based on the player info
            let p1_is_white = white_player.name == "Player1";
            if p1_is_white {
                (Color::White, Color::Black)
            } else {
                (Color::Black, Color::White)
            }
        }
        _ => {
            panic!("Failed to get GameStarted message: {:?}", game_started_1);
        }
    };

    println!(
        "Player 1 is {:?}, Player 2 is {:?}",
        player1_color, player2_color
    );

    // Small delay to ensure both clients are fully ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // The player playing White makes the first move
    println!("White making move e2->e4...");
    if player1_color == Color::White {
        player1.make_move("e2", "e4").await;
    } else {
        player2.make_move("e2", "e4").await;
    }

    // Small delay for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Both should see the move
    let (white_msg, black_msg) = if player1_color == Color::White {
        let w = player1.expect_message(5).await;
        let b = player2.expect_message(5).await;
        (w, b)
    } else {
        let b = player1.expect_message(5).await;
        let w = player2.expect_message(5).await;
        (w, b)
    };

    println!("White received: {:?}", white_msg);
    println!("Black received: {:?}", black_msg);

    assert!(
        matches!(
            white_msg,
            Some(ServerMessage::Public(PublicEvent::MoveMade { .. }))
        ),
        "White should see their move, got: {:?}",
        white_msg
    );

    assert!(
        matches!(
            black_msg,
            Some(ServerMessage::Public(PublicEvent::MoveMade { .. }))
        ),
        "Black should see White's move, got: {:?}",
        black_msg
    );

    // Now it's Black's turn, but White tries to move again
    println!("White trying to move again (should be rejected)...");
    if player1_color == Color::White {
        player1.make_move("d2", "d4").await;
    } else {
        player2.make_move("d2", "d4").await;
    }

    // White should get rejection
    let rejection = if player1_color == Color::White {
        player1.expect_message(2).await
    } else {
        player2.expect_message(2).await
    };

    assert!(
        matches!(
            rejection,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::MoveRejected { .. }
            ))
        ),
        "White should receive MoveRejected for moving out of turn, got: {:?}",
        rejection
    );

    player1.close().await;
    player2.close().await;
    server.stop();
}

#[tokio::test]
async fn test_disconnection_and_reconnection() {
    let server = TestServer::start().await;

    let match_id = server.create_match().await;

    let mut white = TestClient::connect(&server, &match_id).await;
    let mut black = TestClient::connect(&server, &match_id).await;

    // Start game
    white.authenticate("white", "White").await;
    black.authenticate("black", "Black").await;

    // Consume messages
    for _ in 0..4 {
        let _ = white.expect_message(2).await;
        let _ = black.expect_message(2).await;
    }

    // Black disconnects
    black.close().await;

    // White should receive disconnection notification
    let disconnect_msg = white.expect_message(2).await;

    use chers_server_api::server::PlayerConnectionStatus;
    use chers_server_api::{PublicEvent, ServerMessage};

    assert!(
        matches!(
            disconnect_msg,
            Some(ServerMessage::Public(PublicEvent::PlayerStatusChanged {
                status: PlayerConnectionStatus::Disconnected,
                ..
            }))
        ),
        "White should see Black disconnected, got: {:?}",
        disconnect_msg
    );

    // Black reconnects with same token
    let mut black = TestClient::connect(&server, &match_id).await;
    black.authenticate("black", "Black").await;

    // Black should receive StateSync
    let sync_msg = black.expect_message(2).await;

    assert!(
        matches!(
            sync_msg,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::StateSync { .. }
            ))
        ),
        "Black should receive StateSync on reconnect, got: {:?}",
        sync_msg
    );

    // White should see Black reconnected
    let reconnect_msg = white.expect_message(2).await;

    assert!(
        matches!(
            reconnect_msg,
            Some(ServerMessage::Public(PublicEvent::PlayerStatusChanged {
                status: PlayerConnectionStatus::Connected,
                ..
            }))
        ),
        "White should see Black reconnected"
    );

    white.close().await;
    black.close().await;
    server.stop();
}
