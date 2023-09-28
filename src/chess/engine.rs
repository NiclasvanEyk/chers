use super::{
    Board, CastlingRights, Color::White, Coordinate, Move, Piece, Player, State, INITIAL_BOARD,
};

#[derive(Debug)]
pub enum Event {
    Capture {
        at: Coordinate,
        captured: Piece,
        by: Piece,
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
    IllegalMove,
}

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

    pub fn available_moves(&self, state: &State, from: super::Coordinate) -> Vec<Move> {
        let Some(_piece) = state.board[from.y][from.x] else {
            return Vec::new();
        };

        Vec::new()
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

        // TODO: Check for illegal moves

        let mut events = Vec::new();
        if let Some(captured) = state.board[to.y][to.x] {
            events.push(Event::Capture {
                at: to,
                captured,
                by: moved,
            })
        }

        let mut new_board = state.board;
        new_board[from.y][from.x] = None;
        new_board[to.y][to.x] = Some(moved);

        // TODO: Check for checkmate

        Ok((state.new_turn(new_board), events))
    }
}
