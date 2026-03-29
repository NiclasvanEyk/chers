use std::sync::Arc;

use tokio::sync::RwLock;

use crate::matches::{state::Match, MatchId};

pub struct MatchRepository {
    matches: scc::HashMap<MatchId, Arc<RwLock<Match>>>,
}

impl Default for MatchRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchRepository {
    pub fn new() -> Self {
        Self {
            matches: scc::HashMap::new(),
        }
    }

    /// Create a new match in Lobby state
    /// Uses scc's lock-free insert - no global lock on the map
    pub fn create(&self) -> Arc<RwLock<Match>> {
        let id = MatchId::new_v4();
        let match_state = Match::new(id);
        let arc = Arc::new(RwLock::new(match_state));

        // scc::HashMap::insert_sync is lock-free and thread-safe
        // Each entry is independently locked
        let _ = self.matches.insert_sync(id, arc.clone());

        arc
    }

    /// Find a match by ID - returns Arc for shared ownership across handlers
    /// Uses scc::HashMap::read_sync which allows concurrent reads to different keys
    pub fn get(&self, id: MatchId) -> Option<Arc<RwLock<Match>>> {
        // scc::HashMap::read_sync takes a closure that receives (&K, &V)
        // and returns a value. We clone the Arc to increment ref count.
        self.matches.read_sync(&id, |_, v| v.clone())
    }

    /// Remove a match (for cleanup/GC later)
    pub fn remove(&self, id: MatchId) -> Option<Arc<RwLock<Match>>> {
        self.matches.remove_sync(&id).map(|(_, v)| v)
    }
}
