use crate::auth::User;
use crate::communication::command::{Command, CommandResponse, CommandResult, PostGameCommand};
use crate::communication::event::{Event, PostGameEvent, SystemEvent};
use crate::room::Room;

pub fn handle_post_game_command(cmd: Command, room: &mut Room) -> CommandResult {
    match cmd {
        Command::PostGame(PostGameCommand::Reconnect {
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

            let mut events = vec![Event::PostGame(PostGameEvent::PlayerReconnected {
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
        Command::PostGame(PostGameCommand::Leave {
            user,
            connection_id,
        }) => {
            if let Some(existing) = room.players.iter().find(|p| p.id == user.id) {
                if existing.connection_id != connection_id {
                    return CommandResult::rejected("superseded connection");
                }
            }
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
