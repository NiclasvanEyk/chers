use std::sync::Arc;

use axum::extract::{Json, State};

use crate::app::AppState;

#[derive(serde::Serialize)]
pub struct NewMatchResponse {
    id: u32,
}

pub async fn create_new_match(State(state): State<Arc<AppState>>) -> Json<NewMatchResponse> {
    let mut matches = state.matches.lock().await;
    let game = matches.start();
    return Json(NewMatchResponse { id: game.id });
}
