use std::fmt::Display;

use super::{Board, Color, Piece, State, BOARD_SIZE};

// An offset relative to the top left (0,0) from white's view
//
// Due to the [`Board`] being layed out as an array
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self.x {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '?',
        };

        let y = 8 - self.y;

        write!(f, "{}{}", x.to_uppercase(), y)
    }
}

#[derive(Debug)]
pub enum CoordinateParserError {
    Empty,
    MissingColumn,
    InvalidColumn,
    MissingRow,
    InvalidRow,
}

impl Coordinate {
    /// Parses a coordinate in algebraic notation
    pub fn algebraic(input: &str) -> Result<Self, CoordinateParserError> {
        let normalized = input.trim();
        if normalized.is_empty() {
            return Err(CoordinateParserError::Empty);
        }

        let characters: Vec<char> = normalized.chars().take(2).collect();

        let x = match characters.first() {
            Some(column) => match column.to_ascii_lowercase() {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
                _ => return Err(CoordinateParserError::InvalidColumn),
            },
            None => return Err(CoordinateParserError::MissingColumn),
        };

        let y = match characters.get(1) {
            Some(row) => match row.to_ascii_lowercase() {
                '1' => 7,
                '2' => 6,
                '3' => 5,
                '4' => 4,
                '5' => 3,
                '6' => 2,
                '7' => 1,
                '8' => 0,
                _ => return Err(CoordinateParserError::InvalidRow),
            },
            None => return Err(CoordinateParserError::MissingRow),
        };

        Ok(Coordinate { x, y })
    }

    pub fn horizontal(&self, amount: isize) -> Option<Self> {
        let Some(x) = self.x.checked_add_signed(amount) else {
            return None;
        };

        if x >= BOARD_SIZE {
            return None;
        }

        Some(Self { x, y: self.y })
    }

    pub fn vertical(&self, amount: isize) -> Option<Self> {
        let Some(y) = self.y.checked_add_signed(amount) else {
            return None;
        };

        if y >= BOARD_SIZE {
            return None;
        }

        Some(Self { x: self.x, y })
    }

    pub fn diagonal(&self, horizontal: isize, vertical: isize) -> Option<Self> {
        let Some(first) = self.horizontal(horizontal) else {
            return None;
        };

        first.vertical(vertical)
    }

    pub fn forward(&self, color: Color, amount: isize) -> Option<Self> {
        self.vertical(match color {
            super::Color::White => -amount,
            super::Color::Black => amount,
        })
    }

    pub fn up(&self, amount: isize) -> Option<Self> {
        self.vertical(amount)
    }

    pub fn down(&self, amount: isize) -> Option<Self> {
        self.vertical(-amount)
    }

    pub fn right(&self, amount: isize) -> Option<Self> {
        self.horizontal(amount)
    }

    pub fn left(&self, amount: isize) -> Option<Self> {
        self.horizontal(-amount)
    }

    pub fn is_free(&self, board: &Board) -> bool {
        self.piece(board).is_none()
    }

    // TODO: Those two don't really fit in here. Maybe move somewhere else?

    pub fn can_be_moved_to_given(&self, state: &State) -> bool {
        match self.piece(&state.board) {
            None => true,
            Some(piece) => piece.color != state.player,
        }
    }

    pub fn piece(&self, board: &Board) -> Option<Piece> {
        board[self.y][self.x]
    }
}
