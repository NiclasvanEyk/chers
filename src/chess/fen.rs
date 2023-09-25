use std::char;

use super::{empty_board, empty_row, Board, Figure, Piece, Player, Row, State, BOARD_SIZE};

#[derive(Debug)]
pub enum CouldNotParse {
    InvalidNumberOfRows,
    InvalidNumberOfColumns(String),
    InvalidCharacter(char),
}

pub fn parse_state(notation: &str) -> Result<State, CouldNotParse> {
    let board = parse_board(notation)?;

    return Ok(State {
        board,
        player: super::Color::White,
    });
}

pub fn parse_board(notation: &str) -> Result<Board, CouldNotParse> {
    let rows: Vec<&str> = notation.split('/').collect();
    if rows.len() != BOARD_SIZE {
        return Err(CouldNotParse::InvalidNumberOfRows);
    }

    let mut board: Board = empty_board();
    for (index, row) in rows.into_iter().enumerate() {
        board[index] = parse_row(row)?;
    }

    return Ok(board);
}

fn parse_row(row: &str) -> Result<Row, CouldNotParse> {
    let mut pieces: Row = empty_row();
    let mut index = 0;

    for character in row.chars() {
        if index == BOARD_SIZE {
            return Err(CouldNotParse::InvalidNumberOfColumns(String::from(row)));
        }

        if let Some(digit) = character.to_digit(10) {
            for _ in 0..digit {
                if index == BOARD_SIZE {
                    return Err(CouldNotParse::InvalidNumberOfColumns(String::from(row)));
                }

                index += 1;
            }
            continue;
        }

        pieces[index] = Some(parse_piece(character)?);
        index += 1
    }

    if index != BOARD_SIZE {
        return Err(CouldNotParse::InvalidNumberOfColumns(String::from(row)));
    }

    return Ok(pieces);
}

fn parse_piece(character: char) -> Result<Piece, CouldNotParse> {
    let figure = match character.to_lowercase().to_string().as_str() {
        "r" => Figure::Rook,
        "n" => Figure::Knight,
        "b" => Figure::Bishop,
        "q" => Figure::Queen,
        "k" => Figure::King,
        "p" => Figure::Pawn,
        _ => return Err(CouldNotParse::InvalidCharacter(character)),
    };

    return Ok(Piece {
        figure,
        color: owner(character),
        moved: false, // TODO
    });
}

fn owner(character: char) -> Player {
    if character.is_uppercase() {
        return Player::White;
    }

    return Player::Black;
}

#[cfg(test)]
mod tests {
    use crate::chess::Engine;

    use super::*;

    #[test]
    fn it_parses_the_initial_board_state_correctly() {
        let notation = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let parsed = parse_state(notation).unwrap();

        assert_eq!(Engine::new().start(), parsed);
    }
}
