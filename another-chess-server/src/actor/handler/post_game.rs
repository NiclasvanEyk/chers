use crate::auth::User;
use crate::communication::command::{Command, CommandResult, PostGameCommand};
use crate::communication::event::{Event, PostGameEvent};
use crate::room::Room;

pub fn handle_post_game_command(cmd: Command, room: &mut Room) -> CommandResult {
    match cmd {
        Command::PostGame(PostGameCommand::Reconnect { secret }) => {
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
            CommandResult::accepted(Event::PostGame(PostGameEvent::PlayerReconnected { user }))
        }
        Command::PostGame(PostGameCommand::Leave { user }) => {
            room.players.retain(|p| p.id != user.id);
            CommandResult::accepted(Event::PostGame(PostGameEvent::PlayerLeft { user }))
        }
        Command::PostGame(PostGameCommand::OfferRematch { user }) => {
            CommandResult::accepted(Event::PostGame(PostGameEvent::RematchOffered { by: user }))
        }
        Command::PostGame(PostGameCommand::AcceptRematch { user }) => {
            CommandResult::accepted(Event::PostGame(PostGameEvent::RematchAccepted { by: user }))
        }
        Command::PostGame(PostGameCommand::DeclineRematch { user }) => {
            CommandResult::accepted(Event::PostGame(PostGameEvent::RematchDeclined { by: user }))
        }
        _ => CommandResult::rejected("command not valid in post-game phase"),
    }
}
