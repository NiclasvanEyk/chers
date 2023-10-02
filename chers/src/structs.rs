use super::Coordinate;

#[derive(Debug, PartialEq, Clone, Copy)]
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
}

pub type Player = Color;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Figure {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PromotedFigure {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl PromotedFigure {
    pub fn to_figure(&self) -> Figure {
        match (self) {
            PromotedFigure::King => Figure::King,
            PromotedFigure::Queen => Figure::Queen,
            PromotedFigure::Rook => Figure::Rook,
            PromotedFigure::Bishop => Figure::Bishop,
            PromotedFigure::Knight => Figure::Knight,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: Coordinate,
    pub to: Coordinate,
    pub promotion: Option<PromotedFigure>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, Clone)]
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
    pub fn new_turn(&self, new_board: Board) -> Self {
        Self {
            player: self.player.switch(),
            board: new_board,
            castling_rights: self.castling_rights, // TODO
            en_passant_target: None,               // Checked afterwards
            halfmove_clock: self.halfmove_clock,   // TODO
            fullmove_number: match self.player {
                Color::White => self.fullmove_number,
                Color::Black => self.fullmove_number + 1,
            },
        }
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
