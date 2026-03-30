pub mod handlers;
pub mod matches;

pub use matches::repository::MatchRepository;

use std::sync::Arc;

pub struct AppState {
    pub matches: MatchRepository,
}

pub use axum;
pub use tokio;
