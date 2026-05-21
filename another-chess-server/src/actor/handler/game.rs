use crate::auth::User;
use crate::communication::command::{Command, CommandResult, GameCommand};
use crate::communication::event::{Event, GameEvent};
use crate::room::Room;

pub fn handle_game_command(cmd: Command, room: &mut Room) -> CommandResult {
    match cmd {
        Command::Game(GameCommand::Reconnect { secret }) => {
            let entry = match room.auth.lookup(&secret) {
                Some(e) => e.clone(),
                None => return CommandResult::rejected("unknown secret"),
            };
            let user = User {
                id: entry.user_id,
                name: entry.name,
            };
            if !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("user not in this room");
            }
            CommandResult::accepted(Event::Game(GameEvent::PlayerReconnected { user }))
        }
        Command::Game(GameCommand::Leave { user }) => {
            room.players.retain(|p| p.id != user.id);
            CommandResult::accepted(Event::Game(GameEvent::PlayerLeft { user }))
        }
        Command::Game(GameCommand::MakeMove { user, turn }) => {
            if !room.players.iter().any(|p| p.id == user.id) {
                return CommandResult::rejected("user not in this room");
            }
            CommandResult::accepted(Event::Game(GameEvent::Turn { author: user, turn }))
        }
        _ => CommandResult::rejected("command not valid in game phase"),
    }
}
