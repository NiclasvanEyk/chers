use std::sync::Arc;

use axum::extract::{Json, State};

use crate::AppState;

#[derive(serde::Serialize)]
pub struct NewMatchResponse {
    id: String, // UUID as string
}

pub async fn create_new_match(State(state): State<Arc<AppState>>) -> Json<NewMatchResponse> {
    let match_arc = state.matches.create();
    let match_guard = match_arc.read().await;

    Json(NewMatchResponse {
        id: match_guard.id.to_string(),
    })
}
