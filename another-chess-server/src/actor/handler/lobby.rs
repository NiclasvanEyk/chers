use crate::auth::User;
use crate::communication::command::{Command, CommandResponse, CommandResult, LobbyCommand};
use crate::communication::event::{Event, GameEvent, LobbyEvent, SystemEvent};
use crate::room::{MAX_PLAYERS, Phase, Room};

pub fn handle_lobby_command(cmd: Command, room: &mut Room) -> CommandResult {
    match cmd {
        Command::Lobby(LobbyCommand::Join {
            secret,
            name,
            connection_id,
        }) => {
            let entry = match room.auth.authenticate(&secret, &name) {
                Some(e) => e.clone(),
                None => return CommandResult::rejected("room is full"),
            };

            let user = User {
                id: entry.user_id,
                name: entry.name,
                connection_id: connection_id.clone(),
            };

            // Room must have capacity
            if room.players.len() >= MAX_PLAYERS && !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("room is full");
            }

            // Check if this user is already in the room
            if let Some(existing) = room.players.iter_mut().find(|p| p.id == user.id) {
                let old_connection_id = existing.connection_id.clone();
                existing.connection_id = connection_id.clone();

                let mut events = vec![Event::Lobby(LobbyEvent::PlayerJoined {
                    user: user.clone(),
                })];

                if old_connection_id != connection_id {
                    events.push(Event::System(SystemEvent::ConnectionSuperseded {
                        user: user.clone(),
                        old_connection_id,
                    }));
                }

                return CommandResult {
                    response: CommandResponse::Accepted,
                    events,
                };
            }

            room.players.push(user.clone());
            CommandResult::accepted(Event::Lobby(LobbyEvent::PlayerJoined { user }))
        }
        Command::Lobby(LobbyCommand::Leave {
            user,
            connection_id,
        }) => {
            if let Some(existing) = room.players.iter().find(|p| p.id == user.id) {
                if existing.connection_id != connection_id {
                    return CommandResult::rejected("superseded connection");
                }
            }
            room.players.retain(|p| p.id != user.id);
            if let Phase::Lobby { ready_players } = &mut room.phase {
                ready_players.retain(|id| id != &user.id);
            }
            CommandResult::accepted(Event::Lobby(LobbyEvent::PlayerLeft { user }))
        }
        Command::Lobby(LobbyCommand::ChangeName { user, new_name }) => {
            if let Some(player) = room.players.iter_mut().find(|p| p.id == user.id) {
                player.name = new_name;
            }

            let updated_user = room
                .players
                .iter()
                .find(|p| p.id == user.id)
                .cloned()
                .unwrap_or(user);
            CommandResult::accepted(Event::Lobby(LobbyEvent::PlayerNameChanged {
                user: updated_user,
            }))
        }
        Command::Lobby(LobbyCommand::ChangeReady { user, is_ready }) => {
            let should_transition = if let Phase::Lobby { ready_players } = &mut room.phase {
                if is_ready {
                    if !ready_players.contains(&user.id) {
                        ready_players.push(user.id.clone());
                    }
                } else {
                    ready_players.retain(|id| id != &user.id);
                }

                let active_ids: Vec<_> = room.players.iter().map(|p| p.id.clone()).collect();
                ready_players.retain(|id| active_ids.contains(id));

                room.players.len() >= MAX_PLAYERS
                    && room.players.iter().all(|p| ready_players.contains(&p.id))
            } else {
                false
            };

            if should_transition {
                // Randomly assign colors (50/50 chance)
                let (white, black) = if rand::random::<bool>() {
                    (room.players[0].clone(), room.players[1].clone())
                } else {
                    (room.players[1].clone(), room.players[0].clone())
                };
                let game = chers::Game::new();
                room.phase = Phase::Game {
                    state: game.start(),
                    white_player_id: white.id.clone(),
                    black_player_id: black.id.clone(),
                };
                CommandResult {
                    response: CommandResponse::Accepted,
                    events: vec![
                        Event::Lobby(LobbyEvent::PlayerReadyChanged { user, is_ready }),
                        Event::Game(GameEvent::GameStarted { white, black }),
                    ],
                }
            } else {
                CommandResult::accepted(Event::Lobby(LobbyEvent::PlayerReadyChanged {
                    user,
                    is_ready,
                }))
            }
        }
        _ => CommandResult::rejected("command not valid in lobby phase"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::command::GameCommand;

    fn room_with_auth() -> Room {
        let mut room = Room::new("test-room".into());
        room.auth
            .insert("alice-secret", "user-a".to_string(), "Alice");
        room.auth.insert("bob-secret", "user-b".to_string(), "Bob");
        room
    }

    fn user_a() -> User {
        User {
            id: "user-a".into(),
            name: "Alice".into(),
            connection_id: "conn-a".into(),
        }
    }

    fn user_b() -> User {
        User {
            id: "user-b".into(),
            name: "Bob".into(),
            connection_id: "conn-b".into(),
        }
    }

    #[test]
    fn join_adds_player() {
        let mut room = room_with_auth();

        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-1".into(),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Accepted));
        assert_eq!(result.events.len(), 1);
        assert!(
            matches!(&result.events[0], Event::Lobby(LobbyEvent::PlayerJoined { user: u }) if u.id == "user-a")
        );
        assert_eq!(room.players.len(), 1);
        assert_eq!(room.players[0].id, "user-a");
        assert_eq!(room.players[0].connection_id, "conn-1");
    }

    #[test]
    fn join_updates_connection_id_on_rejoin() {
        let mut room = room_with_auth();

        // First join
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-1".into(),
            }),
            &mut room,
        );

        // Rejoin with new connection_id — should update, not reject
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-2".into(),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Accepted));
        // Should have PlayerJoined + ConnectionSuperseded
        assert_eq!(result.events.len(), 2);
        assert!(matches!(
            &result.events[0],
            Event::Lobby(LobbyEvent::PlayerJoined { .. })
        ));
        assert!(
            matches!(&result.events[1], Event::System(SystemEvent::ConnectionSuperseded { old_connection_id, .. }) if old_connection_id == "conn-1")
        );
        assert_eq!(room.players.len(), 1);
        assert_eq!(room.players[0].connection_id, "conn-2");
    }

    #[test]
    fn join_rejects_full_room() {
        let mut room = room_with_auth();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-1".into(),
            }),
            &mut room,
        );
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "bob-secret".into(),
                name: "Bob".into(),
                connection_id: "conn-2".into(),
            }),
            &mut room,
        );
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "charlie-secret".into(),
                name: "Charlie".into(),
                connection_id: "conn-3".into(),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Rejected { .. }));
    }

    #[test]
    fn leave_removes_player() {
        let mut room = room_with_auth();
        let user = user_a();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );
        assert!(!room.players.is_empty());

        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::Leave {
                user: user.clone(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Accepted));
        assert!(matches!(
            &result.events[0],
            Event::Lobby(LobbyEvent::PlayerLeft { .. })
        ));
        assert!(room.players.is_empty());
    }

    #[test]
    fn leave_removes_ready_state() {
        let mut room = room_with_auth();
        let user = user_a();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user: user.clone(),
                is_ready: true,
            }),
            &mut room,
        );

        if let Phase::Lobby { ready_players } = &room.phase {
            assert!(ready_players.contains(&user.id));
        } else {
            panic!("expected lobby phase");
        }

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Leave {
                user,
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );

        if let Phase::Lobby { ready_players } = &room.phase {
            assert!(ready_players.is_empty());
        } else {
            panic!("expected lobby phase");
        }
    }

    #[test]
    fn change_name_updates_player() {
        let mut room = room_with_auth();
        let user = user_a();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeName {
                user,
                new_name: "NewName".into(),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Accepted));
        assert_eq!(room.players[0].name, "NewName");
    }

    #[test]
    fn change_ready_toggles_ready_players() {
        let mut room = room_with_auth();
        let user = user_a();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );

        // Ready up
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user: user.clone(),
                is_ready: true,
            }),
            &mut room,
        );
        assert!(matches!(result.response, CommandResponse::Accepted));
        if let Phase::Lobby { ready_players } = &room.phase {
            assert!(ready_players.contains(&user.id));
        } else {
            panic!("expected lobby phase");
        }

        // Un-ready
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user: user.clone(),
                is_ready: false,
            }),
            &mut room,
        );
        assert!(matches!(result.response, CommandResponse::Accepted));
        if let Phase::Lobby { ready_players } = &room.phase {
            assert!(ready_players.is_empty());
        } else {
            panic!("expected lobby phase");
        }
    }

    #[test]
    fn change_ready_cleans_stale_entries() {
        let mut room = room_with_auth();

        if let Phase::Lobby { ready_players } = &mut room.phase {
            ready_players.push("ghost".into());
        }

        let user = user_a();
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user,
                is_ready: true,
            }),
            &mut room,
        );

        if let Phase::Lobby { ready_players } = &room.phase {
            assert!(!ready_players.contains(&"ghost".into()));
            assert_eq!(ready_players.len(), 1);
        } else {
            panic!("expected lobby phase");
        }
    }

    #[test]
    fn both_ready_triggers_game_started() {
        let mut room = room_with_auth();

        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "alice-secret".into(),
                name: "Alice".into(),
                connection_id: "conn-a".into(),
            }),
            &mut room,
        );
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::Join {
                secret: "bob-secret".into(),
                name: "Bob".into(),
                connection_id: "conn-b".into(),
            }),
            &mut room,
        );
        let _ = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user: user_b(),
                is_ready: true,
            }),
            &mut room,
        );
        let result = handle_lobby_command(
            Command::Lobby(LobbyCommand::ChangeReady {
                user: user_a(),
                is_ready: true,
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Accepted));
        assert_eq!(result.events.len(), 2);
        assert!(matches!(
            &result.events[0],
            Event::Lobby(LobbyEvent::PlayerReadyChanged { .. })
        ));
        // Verify both players are assigned to different colors (random assignment)
        assert!(
            matches!(&result.events[1], Event::Game(GameEvent::GameStarted { white, black })
                if (white.id == "user-a" && black.id == "user-b") || (white.id == "user-b" && black.id == "user-a")
            ),
            "expected both players to be assigned to different colors randomly"
        );
        assert!(
            matches!(room.phase, Phase::Game { state: _, white_player_id, black_player_id }
                if (&white_player_id == "user-a" && &black_player_id == "user-b")
                    || (&white_player_id == "user-b" && &black_player_id == "user-a")
            )
        );
    }

    #[test]
    fn non_lobby_command_is_rejected() {
        let mut room = room_with_auth();

        let result = handle_lobby_command(
            Command::Game(GameCommand::MakeMove {
                user: user_a(),
                move_: chers::Move::simple(
                    chers::Coordinate::new(4, 6), // E2
                    chers::Coordinate::new(4, 4), // E4
                ),
            }),
            &mut room,
        );

        assert!(matches!(result.response, CommandResponse::Rejected { .. }));
    }
}
