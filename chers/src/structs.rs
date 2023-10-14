use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::Coordinate;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn switch(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn other(self) -> Color {
        self.switch()
    }

    pub fn owns(self, piece: Piece) -> bool {
        self == piece.color
    }
}

pub type Player = Color;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Figure {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PromotedFigure {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PromotedFigure {
    pub fn to_figure(&self) -> Figure {
        match self {
            PromotedFigure::Queen => Figure::Queen,
            PromotedFigure::Rook => Figure::Rook,
            PromotedFigure::Bishop => Figure::Bishop,
            PromotedFigure::Knight => Figure::Knight,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Move {
    pub from: Coordinate,
    pub to: Coordinate,
    pub promotion: Option<PromotedFigure>,
}

#[wasm_bindgen]
impl Move {
    #[wasm_bindgen(constructor)]
    pub fn new(from: Coordinate, to: Coordinate, promotion: Option<PromotedFigure>) -> Move {
        Self {
            from,
            to,
            promotion,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Piece {
    pub color: Color,
    pub figure: Figure,
}

impl Piece {
    pub const fn new(color: Color, figure: Figure) -> Self {
        Self { color, figure }
    }

    pub const fn black(figure: Figure) -> Self {
        Self::new(Color::Black, figure)
    }

    pub const fn white(figure: Figure) -> Self {
        Self::new(Color::White, figure)
    }

    pub fn belongs_to(&self, player: Player) -> bool {
        self.color == player
    }
}

pub const BOARD_SIZE: usize = 8;

pub type Row = [Option<Piece>; BOARD_SIZE];
pub type Board = [Row; BOARD_SIZE];

pub const fn empty_row() -> Row {
    [None, None, None, None, None, None, None, None]
}

pub const fn empty_board() -> Board {
    [
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
    ]
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CastlingRights {
    white: CastleDirections,
    black: CastleDirections,
}

impl CastlingRights {
    pub const fn all() -> Self {
        Self {
            white: CastleDirections::both(),
            black: CastleDirections::both(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CastleDirections {
    queen_side: bool,
    king_side: bool,
}

impl CastleDirections {
    pub const fn both() -> Self {
        Self {
            queen_side: true,
            king_side: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub player: Player,
    pub board: Board,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<Coordinate>,
    pub halfmove_clock: u8,
    pub fullmove_number: u8,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        if self.player != other.player {
            return false;
        }

        if self.castling_rights != other.castling_rights {
            return false;
        }

        if self.en_passant_target != other.en_passant_target {
            return false;
        }

        if self.halfmove_clock != other.halfmove_clock {
            return false;
        }

        if self.fullmove_number != other.fullmove_number {
            return false;
        }

        if self.board.len() != other.board.len() {
            return false;
        }

        for (row_index, row) in self.board.into_iter().enumerate() {
            let other_row = other.board[row_index];
            if row.len() != other_row.len() {
                return false;
            }

            for (col_index, piece) in row.into_iter().enumerate() {
                let other_piece = other_row[col_index];
                if piece != other_piece {
                    return false;
                }
            }
        }

        true
    }
}

impl State {
    pub fn new_turn(
        &self,
        new_board: Board,
        moved: Figure,
        r#move: Move,
        did_capture: bool,
    ) -> Self {
        let from = r#move.from;
        let to = r#move.to;

        Self {
            player: self.player.switch(),
            board: new_board,
            castling_rights: self.castling_rights, // TODO
            en_passant_target: match moved == Figure::Pawn && from.y.abs_diff(to.y) == 2 {
                true => Some(r#move.to),
                false => None,
            },
            halfmove_clock: match did_capture || moved == Figure::Pawn {
                true => 0,
                false => self.halfmove_clock + 1,
            },
            fullmove_number: match self.player {
                Color::White => self.fullmove_number,
                Color::Black => self.fullmove_number + 1,
            },
        }
    }

    pub fn opponent(&self) -> Player {
        self.player.other()
    }
}

pub const INITIAL_BOARD: Board = [
    [
        Some(Piece::black(Figure::Rook)),
        Some(Piece::black(Figure::Knight)),
        Some(Piece::black(Figure::Bishop)),
        Some(Piece::black(Figure::Queen)),
        Some(Piece::black(Figure::King)),
        Some(Piece::black(Figure::Bishop)),
        Some(Piece::black(Figure::Knight)),
        Some(Piece::black(Figure::Rook)),
    ],
    [
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
        Some(Piece::black(Figure::Pawn)),
    ],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
        Some(Piece::white(Figure::Pawn)),
    ],
    [
        Some(Piece::white(Figure::Rook)),
        Some(Piece::white(Figure::Knight)),
        Some(Piece::white(Figure::Bishop)),
        Some(Piece::white(Figure::Queen)),
        Some(Piece::white(Figure::King)),
        Some(Piece::white(Figure::Bishop)),
        Some(Piece::white(Figure::Knight)),
        Some(Piece::white(Figure::Rook)),
    ],
];

pub fn cells(board: &Board) -> Vec<(Coordinate, Option<Piece>)> {
    let mut cells = Vec::new();

    for (y, row) in board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            cells.push((Coordinate { x, y }, *cell));
        }
    }

    cells
}

pub fn pieces(board: &Board) -> Vec<(Coordinate, Piece)> {
    cells(board)
        .into_iter()
        .filter_map(|(coordinate, contents)| contents.map(|piece| (coordinate, piece)))
        .collect()
}
