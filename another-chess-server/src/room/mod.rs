use std::collections::HashMap;

use crate::auth::{User, UserId};

pub mod storage;

pub type RoomId = String;

/// Phase-independent container for players, spectators, etc.
///
/// Basically everything exists in the context of a [Room].
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    Lobby { ready_players: Vec<UserId> },
    Game,
    PostGame { winner: UserId },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub phase: Phase,
    pub players: Vec<User>,
    pub auth: RoomAuth,
}

/// Per-room authentication state, persisted alongside the room.
///
/// Maps secrets → user identities. Unknown secrets are registered on first
/// use, keeping the protocol simple (no pre-registration needed).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RoomAuth {
    entries: HashMap<String, AuthEntry>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthEntry {
    pub user_id: UserId,
    pub name: String,
}

impl RoomAuth {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Resolve a secret to a user identity.
    ///
    /// If the secret is known the existing entry is returned (reconnect).
    /// Otherwise a new [`UserId`] is generated and a fresh entry is created,
    /// up to [`MAX_PLAYERS`] total entries.
    pub fn authenticate(&mut self, secret: &str, name: &str) -> Option<&AuthEntry> {
        // Reconnecting user — secret already known
        if self.entries.contains_key(secret) {
            return self.entries.get(secret);
        }

        // New user — only allow if we have capacity
        if self.entries.len() >= MAX_PLAYERS {
            return None;
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        self.entries.insert(
            secret.to_owned(),
            AuthEntry {
                user_id,
                name: name.to_owned(),
            },
        );
        self.entries.get(secret)
    }

    /// Look up a secret without creating a new entry (used for reconnection).
    pub fn lookup(&self, secret: &str) -> Option<&AuthEntry> {
        self.entries.get(secret)
    }

    /// Directly insert an entry (useful for tests and pre-seeding).
    pub fn insert(&mut self, secret: &str, user_id: UserId, name: &str) {
        self.entries.insert(
            secret.to_owned(),
            AuthEntry {
                user_id,
                name: name.to_owned(),
            },
        );
    }
}

pub const MAX_PLAYERS: usize = 2;

impl Room {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            phase: Phase::Lobby {
                ready_players: Vec::new(),
            },
            players: Vec::with_capacity(MAX_PLAYERS),
            auth: RoomAuth::new(),
        }
    }
}
