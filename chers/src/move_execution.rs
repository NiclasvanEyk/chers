use crate::{
    check::{self, checks, mates},
    moves_available::autocomplete_to,
    Color, Coordinate, Event, Figure, Move, Piece, State, BOARD_SIZE,
};

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

pub fn move_piece(state: &State, r#move: Move) -> Result<(State, Vec<Event>), CantMovePiece> {
    let from = r#move.from;
    let to = r#move.to;

    let Some(moved) = state.board[from.y][from.x] else {
        return Err(CantMovePiece::NoPieceToMove)
    };

    if moved.color != state.player {
        return Err(CantMovePiece::ItBelongsToOtherPlayer);
    }

    let legal = autocomplete_to(state, from);
    if !legal.contains(&to) {
        return Err(CantMovePiece::IllegalMove {
            attempted: r#move,
            legal,
        });
    }

    let mut events = Vec::new();
    let mut new_board = state.board;

    let mut did_capture = false;
    if let Some(captured) = to.piece(&state.board) {
        did_capture = true;
        events.push(Event::Capture {
            at: to,
            captured,
            by: moved,
        });
    } else if let Some(en_passant) = state.en_passant_target {
        let origin = en_passant.backward(state.player.other(), 1).unwrap();
        if origin == to {
            new_board[en_passant.y][en_passant.x] = None;
            did_capture = true;
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

    let checking_state = State {
        board: new_board,
        ..*state
    };

    let checking_pieces = checks(&checking_state);
    if !checking_pieces.is_empty() {
        events.push(Event::Check {
            by: checking_pieces,
        });

        if mates(&checking_state) {
            events.push(Event::Mate);
        }
    }

    let new_state = state.new_turn(new_board, moved.figure, r#move, did_capture);

    // TODO: Enforce 50 turn limit using halfmove clock

    Ok((new_state, events))
}

fn requires_promotion(state: &State, piece: Piece, to: Coordinate) -> bool {
    let board_end = match state.player {
        Color::White => 0,
        Color::Black => BOARD_SIZE - 1,
    };

    piece.figure == Figure::Pawn && to.y == board_end
}
