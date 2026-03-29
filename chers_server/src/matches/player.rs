use jiff::Timestamp;
use tokio::sync::broadcast;

use chers_server_api::PrivateEvent;

#[derive(Clone)]
pub struct PlayerSlot {
    pub name: String,
    pub token: String,
    pub connected: bool,
    pub last_seen_at: Timestamp,
    pub tx: Option<broadcast::Sender<PrivateEvent>>,
}

#[derive(Clone, Debug)]
pub struct PlayerInfo {
    pub name: String,
    pub connected: bool,
    pub token: String,
    pub disconnected_at: Option<Timestamp>,
}
