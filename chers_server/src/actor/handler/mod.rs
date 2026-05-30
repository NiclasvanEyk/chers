use chers::Color;

use crate::auth::User;
use crate::communication::command::{
    Command, CommandResponse, CommandResult, GameCommand, LobbyCommand, PostGameCommand,
    RoomStateMirror,
};
use crate::communication::event::GameEndReason;
use crate::room::{Phase, Room};

use super::handler::{
    game::handle_game_command, lobby::handle_lobby_command, post_game::handle_post_game_command,
};

mod lobby;

mod game;

mod post_game;

pub fn handle_command(cmd: Command, room: &mut Room) -> CommandResult {
    // Handle universal commands first
    match &cmd {
        Command::RequestState { user } => return build_state_mirror(user.clone(), room),
        Command::Leave {
            user,
            connection_id,
        } => {
            let user = user.clone();
            let connection_id = connection_id.clone();
            return match room.phase {
                Phase::Lobby { .. } => handle_lobby_command(
                    Command::Lobby(LobbyCommand::Leave {
                        user,
                        connection_id,
                    }),
                    room,
                ),
                Phase::Game { .. } => handle_game_command(
                    Command::Game(GameCommand::Leave {
                        user,
                        connection_id,
                    }),
                    room,
                ),
                Phase::PostGame { .. } => handle_post_game_command(
                    Command::PostGame(PostGameCommand::Leave {
                        user,
                        connection_id,
                    }),
                    room,
                ),
            };
        }
        _ => {}
    }

    // Phase-specific dispatch
    match room.phase {
        Phase::Lobby { .. } => handle_lobby_command(cmd, room),
        Phase::Game { .. } => handle_game_command(cmd, room),
        Phase::PostGame { .. } => handle_post_game_command(cmd, room),
    }
}

/// Resolve the current user data from the given player list.
///
/// The command carries a stale [`User`] snapshot (the name the client
/// authenticated with). Always prefer the authoritative data stored in
/// `room.players` or the per-phase player snapshot.
fn resolve_user<'a>(players: &'a [User], cmd_user: &'a User) -> &'a User {
    players
        .iter()
        .find(|p| p.id == cmd_user.id)
        .unwrap_or(cmd_user)
}

/// Build a personalized state mirror for the requesting user.
fn build_state_mirror(user: User, room: &Room) -> CommandResult {
    let state_mirror = match &room.phase {
        Phase::Lobby { ready_players } => {
            let you = resolve_user(&room.players, &user).clone();
            let opponent = room.players.iter().find(|p| p.id != user.id).cloned();

            let you_are_ready = ready_players.contains(&user.id);
            let opponent_is_ready = opponent
                .as_ref()
                .map(|op| ready_players.contains(&op.id))
                .unwrap_or(false);

            RoomStateMirror::Lobby {
                you,
                opponent,
                you_are_ready,
                opponent_is_ready,
            }
        }

        Phase::Game {
            state,
            white_player_id,
            black_player_id,
            move_history,
        } => {
            let you = resolve_user(&room.players, &user).clone();
            let (your_color, opponent_color, opponent) = if user.id == *white_player_id {
                let opponent = room
                    .players
                    .iter()
                    .find(|p| p.id == *black_player_id)
                    .cloned()
                    .expect("black player should exist");
                (Color::White, Color::Black, opponent)
            } else {
                let opponent = room
                    .players
                    .iter()
                    .find(|p| p.id == *white_player_id)
                    .cloned()
                    .expect("white player should exist");
                (Color::Black, Color::White, opponent)
            };

            let is_your_turn = state.player == your_color;

            RoomStateMirror::Game {
                you,
                your_color,
                opponent,
                opponent_color,
                board_state: state.clone(),
                is_your_turn,
                move_history: move_history.clone(),
            }
        }

        Phase::PostGame {
            winner,
            reason,
            players,
            final_state,
            white_player_id,
            black_player_id,
            move_history,
        } => {
            let you = resolve_user(players, &user).clone();
            let opponent = players
                .iter()
                .find(|p| p.id != user.id)
                .cloned()
                .expect("opponent should be present in post-game player snapshot");

            let (your_color, opponent_color) = if user.id == *white_player_id {
                (Color::White, Color::Black)
            } else {
                (Color::Black, Color::White)
            };

            let winner_color = if winner.as_ref() == Some(white_player_id) {
                Some(Color::White)
            } else if winner.as_ref() == Some(black_player_id) {
                Some(Color::Black)
            } else {
                None
            };

            let you_won = winner.as_ref() == Some(&user.id);

            RoomStateMirror::PostGame {
                you,
                your_color,
                opponent,
                opponent_color,
                winner: winner_color,
                you_won,
                reason: reason.clone().unwrap_or(GameEndReason::Checkmate),
                final_board: final_state.clone(),
                move_history: move_history.clone(),
            }
        }
    };

    CommandResult {
        response: CommandResponse::State(state_mirror),
        events: vec![],
    }
}
