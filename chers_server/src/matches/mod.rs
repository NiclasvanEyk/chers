pub type MatchId = uuid::Uuid;

pub mod channels;
pub mod player;
pub mod repository;
pub mod state;

pub use state::Match;

/// Parse a match ID from a string (used in URL path parameters)
pub fn parse_match_id(s: &str) -> Option<MatchId> {
    MatchId::parse_str(s).ok()
}
