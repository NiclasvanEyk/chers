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

    // Player 1 receives LobbyJoined first (waiting in lobby)
    // Both players receive ColorsAssigned and GameStarted (order may vary due to tokio::select!)
    use chers_server_api::{PublicEvent, ServerMessage};

    // Player 1: should receive LobbyJoined first (they're waiting)
    let lobby1 = player1.expect_message(5).await;
    println!("Player 1 lobby: {:?}", lobby1);
    assert!(
        matches!(
            lobby1,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::LobbyJoined { .. }
            ))
        ),
        "Player 1 should receive LobbyJoined, got: {:?}",
        lobby1
    );

    // Player 2: should receive LobbyJoined when they join
    let lobby2 = player2.expect_message(5).await;
    println!("Player 2 lobby: {:?}", lobby2);
    assert!(
        matches!(
            lobby2,
            Some(ServerMessage::Private(
                chers_server_api::PrivateEvent::LobbyJoined { .. }
            ))
        ),
        "Player 2 should receive LobbyJoined, got: {:?}",
        lobby2
    );

    // Both players set ready=true
    player1.ready(true).await;
    player2.ready(true).await;

    // Both should receive PlayerReady events
    let ready1 = player1.expect_message(5).await;
    let ready2 = player1.expect_message(5).await; // Both events go to both players
    println!("Player 1 ready events: {:?}, {:?}", ready1, ready2);

    let ready3 = player2.expect_message(5).await;
    let ready4 = player2.expect_message(5).await;
    println!("Player 2 ready events: {:?}, {:?}", ready3, ready4);

    // Expect Countdown events (5, 4, 3, 2, 1)
    for player in [&mut player1, &mut player2] {
        for expected_seconds in [5u8, 4, 3, 2, 1] {
            let msg = player.expect_message(5).await;
            println!("Countdown {}: {:?}", expected_seconds, msg);
            assert!(
                matches!(
                    msg,
                    Some(ServerMessage::Public(PublicEvent::Countdown { seconds })) if seconds == expected_seconds
                ),
                "Expected Countdown({}), got: {:?}", expected_seconds, msg
            );
        }
    }

    // Collect remaining messages: GameStarting, ColorsAssigned, GameStarted
    // These may arrive in different order
    let mut player1_events = vec![];
    let mut player2_events = vec![];
    
    for _ in 0..3 {
        player1_events.push(player1.expect_message(5).await);
        player2_events.push(player2.expect_message(5).await);
    }
    
    println!("Player 1 events: {:?}", player1_events);
    println!("Player 2 events: {:?}", player2_events);
    
    // Verify both players received: GameStarting, ColorsAssigned, GameStarted
    for (events, player_name) in [(&player1_events, "Player 1"), (&player2_events, "Player 2")] {
        let has_starting = events.iter().any(|m| matches!(
            m, Some(ServerMessage::Public(PublicEvent::GameStarting))
        ));
        let has_colors = events.iter().any(|m| matches!(
            m, Some(ServerMessage::Private(chers_server_api::PrivateEvent::ColorsAssigned { .. }))
        ));
        let has_started = events.iter().any(|m| matches!(
            m, Some(ServerMessage::Public(PublicEvent::GameStarted { .. }))
        ));
        
        assert!(has_starting, "{} should receive GameStarting", player_name);
        assert!(has_colors, "{} should receive ColorsAssigned", player_name);
        assert!(has_started, "{} should receive GameStarted", player_name);
    }

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

    // Both receive LobbyJoined
    let _ = player1.expect_message(5).await; // LobbyJoined
    let _ = player2.expect_message(5).await; // LobbyJoined

    // Both players set ready=true
    player1.ready(true).await;
    player2.ready(true).await;

    // Collect all lobby events: 2x PlayerReady, 5x Countdown, 1x GameStarting, 2x (ColorsAssigned or GameStarted)
    // For simplicity, just consume messages until we get GameStarted
    use chers_server_api::{Color, PublicEvent, ServerMessage};

    let game_started_1 = wait_for_game_started(&mut player1).await;
    let game_started_2 = wait_for_game_started(&mut player2).await;

    // Extract which player is playing which color from GameStarted
    let (player1_color, player2_color) = match (game_started_1, game_started_2) {
        (
            PublicEvent::GameStarted {
                white_player,
                black_player,
                ..
            },
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
            panic!("Failed to get GameStarted message");
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

    // Both receive LobbyJoined
    let _ = white.expect_message(2).await;
    let _ = black.expect_message(2).await;

    // Both ready
    white.ready(true).await;
    black.ready(true).await;

    // Wait for game to start (ready events, countdown, game starting)
    use chers_server_api::{PublicEvent, ServerMessage};
    
    // Consume ready events
    for _ in 0..4 {
        let _ = white.expect_message(2).await;
        let _ = black.expect_message(2).await;
    }
    
    // Consume countdown (5 seconds)
    for _ in 0..5 {
        let _ = white.expect_message(2).await;
        let _ = black.expect_message(2).await;
    }
    
    // Consume GameStarting, ColorsAssigned, GameStarted
    for _ in 0..3 {
        let _ = white.expect_message(2).await;
        let _ = black.expect_message(2).await;
    }

    // Black disconnects
    black.close().await;

    // White should receive disconnection notification
    let disconnect_msg = white.expect_message(2).await;

    use chers_server_api::server::PlayerConnectionStatus;

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

// Helper function to wait for GameStarted event during lobby/ready flow
async fn wait_for_game_started(
    client: &mut common::TestClient,
) -> chers_server_api::PublicEvent {
    use chers_server_api::{PublicEvent, PrivateEvent, ServerMessage};
    
    // Keep consuming messages until we find GameStarted
    // Events in order: LobbyJoined, PlayerReady x2, Countdown x5, GameStarting, ColorsAssigned, GameStarted
    let mut found_game_started = None;
    let mut found_colors_assigned = false;
    
    for _ in 0..20 {
        let msg = client.expect_message(10).await;
        
        match msg {
            Some(ServerMessage::Public(PublicEvent::GameStarted { .. })) => {
                if let Some(ServerMessage::Public(event)) = msg {
                    found_game_started = Some(event);
                }
            }
            Some(ServerMessage::Private(PrivateEvent::ColorsAssigned { .. })) => {
                found_colors_assigned = true;
            }
            _ => {}
        }
        
        // Once we have both, we can return
        if found_game_started.is_some() && found_colors_assigned {
            return found_game_started.unwrap();
        }
    }
    
    // If we found GameStarted but not ColorsAssigned, return anyway
    if let Some(event) = found_game_started {
        return event;
    }
    
    panic!("Did not receive GameStarted event within expected number of messages");
}
