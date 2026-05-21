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
        Command::Leave { user, connection_id } => {
            let user = user.clone();
            let connection_id = connection_id.clone();
            return match room.phase {
                Phase::Lobby { .. } => handle_lobby_command(
                    Command::Lobby(LobbyCommand::Leave { user, connection_id }),
                    room,
                ),
                Phase::Game { .. } => handle_game_command(
                    Command::Game(GameCommand::Leave { user, connection_id }),
                    room,
                ),
                Phase::PostGame { .. } => handle_post_game_command(
                    Command::PostGame(PostGameCommand::Leave { user, connection_id }),
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

/// Build a personalized state mirror for the requesting user.
fn build_state_mirror(user: User, room: &Room) -> CommandResult {
    let state_mirror = match &room.phase {
        Phase::Lobby { ready_players } => {
            let opponent = room.players.iter().find(|p| p.id != user.id).cloned();

            let you_are_ready = ready_players.contains(&user.id);
            let opponent_is_ready = opponent
                .as_ref()
                .map(|op| ready_players.contains(&op.id))
                .unwrap_or(false);

            RoomStateMirror::Lobby {
                you: user,
                opponent,
                you_are_ready,
                opponent_is_ready,
            }
        }

        Phase::Game {
            state,
            white_player_id,
            black_player_id,
        } => {
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
                you: user,
                your_color,
                opponent,
                opponent_color,
                board_state: state.clone(),
                is_your_turn,
            }
        }

        Phase::PostGame { winner, reason } => {
            // We need to determine colors from the room's perspective
            // Since we don't track color history in PostGame, we'll look at the previous
            // game state from storage. For now, we'll derive from available info.

            // Find the opponent
            let opponent = room
                .players
                .iter()
                .find(|p| p.id != user.id)
                .cloned()
                .expect("opponent should exist");

            // Try to determine colors - we need to know who was white/black
            // For now, we'll use a placeholder that will be resolved when we
            // check storage or when we add color tracking to PostGame
            // For this implementation, we'll default to the user being White
            // In a real implementation, you'd want to persist color assignments
            let your_color = Color::White;
            let opponent_color = Color::Black;

            let you_won = winner.as_ref() == Some(&user.id);

            // Placeholder - in a full implementation, we'd persist the final board
            // and color assignments when transitioning to PostGame
            let final_board = chers::Game::new().start();

            RoomStateMirror::PostGame {
                you: user,
                your_color,
                opponent,
                opponent_color,
                winner: None, // Would need color of winner
                you_won,
                reason: reason.clone().unwrap_or(GameEndReason::Checkmate),
                final_board,
            }
        }
    };

    CommandResult {
        response: CommandResponse::State(state_mirror),
        events: vec![],
    }
}
