use std::sync::Arc;

use axum::extract::{Json, State};
use tracing::{info, instrument};

use crate::AppState;

#[derive(serde::Serialize)]
pub struct NewMatchResponse {
    id: String, // UUID as string
}

#[instrument(skip(state))]
pub async fn create_new_match(State(state): State<Arc<AppState>>) -> Json<NewMatchResponse> {
    let match_arc = state.matches.create();
    let match_guard = match_arc.read().await;
    let match_id = match_guard.id.to_string();

    info!(match_id = %match_id, "🎮 New match created");

    Json(NewMatchResponse { id: match_id })
}
