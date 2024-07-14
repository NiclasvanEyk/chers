use crate::matches::{Match, MatchId};
use std::sync::Arc;

#[derive(Default)]
pub struct MatchRepository {
    last_id: u32,
    matches: Vec<Arc<Match>>,
}

impl MatchRepository {
    fn generate_id(&mut self) -> MatchId {
        let id = self.last_id.wrapping_add(1);
        self.last_id = id;

        id
    }

    pub fn start(&mut self) -> Arc<Match> {
        let m = Arc::new(Match::new(self.generate_id()));
        self.matches.push(m.clone());

        m
    }

    pub fn find(&self, id: MatchId) -> Option<Arc<Match>> {
        self.matches.iter().find(|m| m.id == id).cloned()
    }
}
