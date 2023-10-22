use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

use crate::{
    move_piece, moves_available::autocomplete_to, CantMovePiece, Coordinate, Engine, Event, Move,
    State,
};

use serde_wasm_bindgen as bridge;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MoveError {
    RequiresPromotion,
    InvalidMove,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Error {
    pub error: MoveError,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MoveResult {
    pub next_state: State,
    pub events: Vec<Event>,
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

    match move_piece(&state, the_move) {
        Err(error) => bridge::to_value(&Error {
            error: match error {
                CantMovePiece::NoPieceToMove => MoveError::InvalidMove,
                CantMovePiece::ItBelongsToOtherPlayer => MoveError::InvalidMove,
                CantMovePiece::RequiresPromotion => MoveError::RequiresPromotion,
                CantMovePiece::IllegalMove {
                    attempted: _,
                    legal: _,
                } => MoveError::InvalidMove,
            },
        })
        .unwrap(),
        Ok((next_state, events)) => bridge::to_value(&MoveResult {
            next_state,
            check: is_check(&events),
            mate: is_mate(&events),
            events,
        })
        .unwrap(),
    }
}

fn is_check(events: &[Event]) -> bool {
    events.iter().any(|x| match x {
        Event::Capture {
            at: _,
            captured: _,
            by: _,
        } => false,
        Event::Move {
            piece: _,
            from: _,
            to: _,
        } => false,
        Event::Promotion { to: _ } => false,
        Event::Check { by: _ } => true,
        Event::Mate => false,
    })
}

fn is_mate(events: &[Event]) -> bool {
    events.iter().any(|x| match x {
        Event::Capture {
            at: _,
            captured: _,
            by: _,
        } => false,
        Event::Move {
            piece: _,
            from: _,
            to: _,
        } => false,
        Event::Promotion { to: _ } => false,
        Event::Check { by: _ } => false,
        Event::Mate => true,
    })
}
