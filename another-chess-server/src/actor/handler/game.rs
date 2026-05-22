use crate::auth::User;
use crate::communication::command::{Command, CommandResponse, CommandResult, GameCommand};
use crate::communication::event::{Event, GameEvent, SystemEvent};
use crate::room::{Phase, Room};

// Re-export from the API crate
pub use chers_server_api::v2::events::GameEndReason;

pub fn handle_game_command(cmd: Command, room: &mut Room) -> CommandResult {
    match cmd {
        Command::Game(GameCommand::Reconnect {
            secret,
            connection_id,
        }) => {
            let entry = match room.auth.lookup(&secret) {
                Some(e) => e.clone(),
                None => return CommandResult::rejected("unknown secret"),
            };
            let user = User {
                id: entry.user_id,
                name: entry.name,
                connection_id: connection_id.clone(),
            };
            if !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("user not in this room");
            }

            let mut events = vec![Event::Game(GameEvent::PlayerReconnected {
                user: user.clone(),
            })];

            // Update connection_id and emit superseded event if changed
            if let Some(existing) = room.players.iter_mut().find(|p| p.id == user.id) {
                if existing.connection_id != connection_id {
                    let old_connection_id = existing.connection_id.clone();
                    existing.connection_id = connection_id;
                    events.push(Event::System(SystemEvent::ConnectionSuperseded {
                        user: user.clone(),
                        old_connection_id,
                    }));
                }
            }

            CommandResult {
                response: CommandResponse::Accepted,
                events,
            }
        }
        Command::Game(GameCommand::Leave {
            user,
            connection_id,
        }) => {
            if let Some(existing) = room.players.iter().find(|p| p.id == user.id) {
                if existing.connection_id != connection_id {
                    return CommandResult::rejected("superseded connection");
                }
            }
            room.players.retain(|p| p.id != user.id);
            CommandResult::accepted(Event::Game(GameEvent::PlayerLeft { user }))
        }
        Command::Game(GameCommand::MakeMove { user, move_ }) => {
            if !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("user not in this room");
            }

            // Validate the move using the chess engine if we're in a game
            if let Phase::Game {
                state,
                white_player_id,
                black_player_id,
            } = &mut room.phase
            {
                // Check if it's this player's turn based on their color assignment
                let is_white = &user.id == white_player_id;
                let is_black = &user.id == black_player_id;

                if !is_white && !is_black {
                    return CommandResult::accepted(Event::Game(GameEvent::MoveRejected {
                        author: user,
                        reason: "you are not a player in this game".to_string(),
                    }));
                }

                let current_player = if is_white {
                    chers::Color::White
                } else {
                    chers::Color::Black
                };

                // Verify it's this player's turn
                if state.player != current_player {
                    return CommandResult::accepted(Event::Game(GameEvent::MoveRejected {
                        author: user,
                        reason: "not your turn".to_string(),
                    }));
                }

                let game = chers::Game::new();

                // Pre-validate the move to provide better error messages
                if !game.is_valid_move(state, move_) {
                    return CommandResult::accepted(Event::Game(GameEvent::MoveRejected {
                        author: user,
                        reason: "illegal move".to_string(),
                    }));
                }

                match game.move_piece(state, move_) {
                    Ok((new_state, events)) => {
                        *state = new_state;

                        // Check for checkmate
                        let is_checkmate = events.iter().any(|e| matches!(e, chers::Event::Mate));

                        if is_checkmate {
                            // Determine winner info before transitioning phase
                            let winner_id = if current_player == chers::Color::White {
                                white_player_id.clone()
                            } else {
                                black_player_id.clone()
                            };
                            let winner_user =
                                room.players.iter().find(|p| p.id == winner_id).cloned();

                            // Transition to PostGame phase
                            room.phase = Phase::PostGame {
                                winner: Some(winner_id),
                                reason: Some(GameEndReason::Checkmate),
                                players: room.players.clone(),
                            };

                            CommandResult {
                                response: CommandResponse::Accepted,
                                events: vec![
                                    Event::Game(GameEvent::Turn {
                                        author: user,
                                        move_,
                                    }),
                                    Event::Game(GameEvent::GameEnded {
                                        winner: winner_user,
                                        reason: GameEndReason::Checkmate,
                                    }),
                                ],
                            }
                        } else {
                            CommandResult::accepted(Event::Game(GameEvent::Turn {
                                author: user,
                                move_,
                            }))
                        }
                    }
                    Err(_) => CommandResult::accepted(Event::Game(GameEvent::MoveRejected {
                        author: user,
                        reason: "invalid move".to_string(),
                    })),
                }
            } else {
                CommandResult::rejected("not in game phase")
            }
        }
        Command::Game(GameCommand::Resign { user }) => {
            if !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("user not in this room");
            }

            if let Phase::Game {
                white_player_id,
                black_player_id,
                ..
            } = &room.phase
            {
                // Determine who is resigning and who wins
                let winner_id = if &user.id == white_player_id {
                    black_player_id.clone()
                } else if &user.id == black_player_id {
                    white_player_id.clone()
                } else {
                    return CommandResult::rejected("you are not a player in this game");
                };

                let winner_user = room.players.iter().find(|p| p.id == winner_id).cloned();

                // Transition to PostGame phase
                room.phase = Phase::PostGame {
                    winner: Some(winner_id),
                    reason: Some(GameEndReason::Resignation),
                    players: room.players.clone(),
                };

                CommandResult {
                    response: CommandResponse::Accepted,
                    events: vec![Event::Game(GameEvent::GameEnded {
                        winner: winner_user,
                        reason: GameEndReason::Resignation,
                    })],
                }
            } else {
                CommandResult::rejected("not in game phase")
            }
        }
        _ => CommandResult::rejected("command not valid in game phase"),
    }
}
