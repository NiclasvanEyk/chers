#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black,
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

#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: Coordinate,
    pub to: Coordinate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub figure: Figure,
    pub moved: bool,
}

impl Piece {
    pub const fn new(color: Color, figure: Figure) -> Self {
        return Self {
            color,
            figure,
            moved: false,
        };
    }

    pub const fn black(figure: Figure) -> Self {
        return Self::new(Color::Black, figure);
    }

    pub const fn white(figure: Figure) -> Self {
        return Self::new(Color::White, figure);
    }
}

pub const BOARD_SIZE: usize = 8;

pub type Row = [Option<Piece>; BOARD_SIZE];
pub type Board = [Row; BOARD_SIZE];

pub const fn empty_row() -> Row {
    return [None, None, None, None, None, None, None, None];
}

pub const fn empty_board() -> Board {
    return [
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
        empty_row(),
    ];
}

#[derive(Debug)]
pub struct State {
    pub player: Player,
    pub board: Board,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        if self.player != other.player {
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

        return true;
    }
}

impl State {
    pub fn new_turn(&self, new_board: Board) -> Self {
        return Self {
            player: self.player.switch(),
            board: new_board,
        };
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
