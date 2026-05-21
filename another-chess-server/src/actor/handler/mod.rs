use crate::communication::command::{Command, CommandResult};
use crate::room::{Phase, Room};

use super::handler::{
    game::handle_game_command, lobby::handle_lobby_command, post_game::handle_post_game_command,
};

mod lobby;

mod game;

mod post_game;

pub fn handle_command(cmd: Command, room: &mut Room) -> CommandResult {
    match room.phase {
        Phase::Lobby { .. } => handle_lobby_command(cmd, room),
        Phase::Game => handle_game_command(cmd, room),
        Phase::PostGame { .. } => handle_post_game_command(cmd, room),
    }
}
