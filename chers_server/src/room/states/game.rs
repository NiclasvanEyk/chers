use axum::extract::ws::WebSocket;
use chers_server_api::ProgressingMatchCommand;

struct ProgressingMatchStateMachine {}

impl ProgressingMatchStateMachine {
    pub fn on_reconnect(self, socket: &mut WebSocket) {
        // Re-authenticate
        // Swap out the previous socket with the new one
        // Send down the current game state
    }

    pub fn on_message(self, command: ProgressingMatchCommand) {}
}
