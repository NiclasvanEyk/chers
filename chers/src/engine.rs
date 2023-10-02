use crate::{Color, Figure, PromotedFigure, BOARD_SIZE};

use super::{
    moves::autocomplete_to, Board, CastlingRights, Color::White, Coordinate, Move, Piece, Player,
    State, INITIAL_BOARD,
};

#[derive(Debug)]
pub enum Event {
    Capture {
        at: Coordinate,
        captured: Piece,
        by: Piece,
    },
    Promotion {
        to: PromotedFigure,
    },
    Check {
        by: Player,
    },
    Mate {
        by: Player,
        board: Board,
    },
}

#[derive(Debug)]
pub enum CantMovePiece {
    NoPieceToMove,
    ItBelongsToOtherPlayer,
    RequiresPromotion,
    IllegalMove {
        attempted: Move,
        legal: Vec<Coordinate>,
    },
}

#[derive(Default)]
pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> State {
        State {
            player: White,
            board: INITIAL_BOARD,
            castling_rights: CastlingRights::all(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn available_moves(&self, state: &State, from: Coordinate) -> Vec<Coordinate> {
        let Some(piece) = state.board[from.y][from.x] else {
            return Vec::new();
        };

        autocomplete_to(state, from, piece)
    }

    pub fn move_piece(
        &self,
        state: &State,
        r#move: Move,
    ) -> Result<(State, Vec<Event>), CantMovePiece> {
        let from = r#move.from;
        let to = r#move.to;

        let Some(moved) = state.board[from.y][from.x] else {
            return Err(CantMovePiece::NoPieceToMove)
        };

        if moved.color != state.player {
            return Err(CantMovePiece::ItBelongsToOtherPlayer);
        }

        let legal = self.available_moves(state, from);
        if !legal.contains(&to) {
            return Err(CantMovePiece::IllegalMove {
                attempted: r#move,
                legal,
            });
        }

        let mut events = Vec::new();
        let mut new_board = state.board;

        if let Some(captured) = to.piece(&state.board) {
            events.push(Event::Capture {
                at: to,
                captured,
                by: moved,
            });
        } else if let Some(en_passant) = state.en_passant_target {
            let origin = en_passant.backward(state.player.other(), 1).unwrap();
            if origin == to {
                new_board[en_passant.y][en_passant.x] = None;
                events.push(Event::Capture {
                    // TODO: Maybe we need to introduce more fields here?
                    at: to,
                    // Save to unwrap, since en_passant target is present
                    captured: en_passant.piece(&state.board).unwrap(),
                    by: moved,
                });
            }
        }

        new_board[from.y][from.x] = None;

        if requires_promotion(state, moved, to) {
            let Some(promoted) = r#move.promotion else {
                return Err(CantMovePiece::RequiresPromotion);
            };

            events.push(Event::Promotion { to: promoted });
            new_board[to.y][to.x] = Some(Piece {
                color: state.player,
                figure: promoted.to_figure(),
            });
        } else {
            new_board[to.y][to.x] = Some(moved);
        }

        // TODO: Check for checkmate

        let mut new_state = state.new_turn(new_board);

        let enables_en_passant = moved.figure == Figure::Pawn && from.y.abs_diff(to.y) == 2;
        if enables_en_passant {
            new_state.en_passant_target = Some(to)
        }

        Ok((new_state, events))
    }
}

fn requires_promotion(state: &State, piece: Piece, to: Coordinate) -> bool {
    let board_end = match state.player {
        Color::White => 0,
        Color::Black => BOARD_SIZE - 1,
    };

    piece.figure == Figure::Pawn && to.y == board_end
}
