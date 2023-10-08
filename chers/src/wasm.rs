use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{move_piece, moves_available::autocomplete_to, Coordinate, Engine, Move, State};

use serde_wasm_bindgen as bridge;

#[derive(Serialize, Deserialize)]
pub struct MoveResult {
    pub next_state: State,
    pub check: bool,
    pub mate: bool,
}

#[wasm_bindgen]
pub fn new_game() -> JsValue {
    let state = Engine::new().start();

    bridge::to_value(&state).unwrap()
}

#[wasm_bindgen]
pub fn available_moves(unsafe_state: JsValue, from: Coordinate) -> JsValue {
    let state: State = bridge::from_value(unsafe_state).unwrap();
    let moves = autocomplete_to(&state, from);

    bridge::to_value(&moves).unwrap()
}

#[wasm_bindgen]
pub fn next_state(unsafe_state: JsValue, the_move: Move) -> JsValue {
    let state: State = bridge::from_value(unsafe_state).unwrap();
    let (next_state, events) = move_piece(&state, the_move).unwrap(); // TODO

    bridge::to_value(&MoveResult {
        next_state,
        check: false, // TODO
        mate: false,  // TODO
    })
    .unwrap()
}
