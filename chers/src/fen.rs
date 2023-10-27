use std::{char, num::ParseIntError};

use super::{
    empty_board, empty_row, Board, CastlingRights, Coordinate, Figure, Piece, Player, Row, State,
    BOARD_SIZE,
};

#[derive(Debug)]
pub enum CouldNotParse {
    InvalidNumberOfParts,
    InvalidPlayerChar(String),
    InvalidNumberOfRows,
    InvalidNumberOfColumns(String),
    InvalidHalfmoveClock(ParseIntError),
    InvalidFullmoveNumber(ParseIntError),
    InvalidPiece(char),
    InvalidEnPassantTarget(String),
}

pub fn parse_state(notation: &str) -> Result<State, CouldNotParse> {
    let parts: Vec<&str> = notation.split(' ').collect();
    if parts.len() != 6 {
        return Err(CouldNotParse::InvalidNumberOfParts);
    }

    Ok(State {
        board: parse_board(parts[0])?,
        player: parse_player(parts[1])?,
        castling_rights: parse_castling_rights(parts[2])?,
        en_passant_target: parse_en_passant_target(parts[3])?,
        halfmove_clock: parse_halfmove_clock(parts[4])?,
        fullmove_number: parse_fullmove_number(parts[5])?,
    })
}

fn parse_halfmove_clock(notation: &str) -> Result<u8, CouldNotParse> {
    match notation.parse::<u8>() {
        Ok(value) => Ok(value),
        Err(err) => Err(CouldNotParse::InvalidHalfmoveClock(err)),
    }
}

fn parse_fullmove_number(notation: &str) -> Result<u8, CouldNotParse> {
    match notation.parse::<u8>() {
        Ok(value) => Ok(value),
        Err(err) => Err(CouldNotParse::InvalidFullmoveNumber(err)),
    }
}

fn parse_en_passant_target(notation: &str) -> Result<Option<Coordinate>, CouldNotParse> {
    if notation.is_empty() || notation == "-" {
        return Ok(None);
    }

    match Coordinate::algebraic(notation) {
        Ok(coordinate) => Ok(Some(coordinate)),
        Err(_) => Err(CouldNotParse::InvalidEnPassantTarget(notation.to_owned())),
    }
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

    Ok(board)
}

fn parse_row(row: &str) -> Result<Row, CouldNotParse> {
    let mut pieces: Row = empty_row();
    let mut index = 0;

    for character in row.chars() {
        if index == BOARD_SIZE {
            return Err(CouldNotParse::InvalidNumberOfColumns(row.to_owned()));
        }

        if let Some(digit) = character.to_digit(10) {
            for _ in 0..digit {
                if index == BOARD_SIZE {
                    return Err(CouldNotParse::InvalidNumberOfColumns(row.to_owned()));
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

    Ok(pieces)
}

fn parse_piece(character: char) -> Result<Piece, CouldNotParse> {
    let figure = match character.to_lowercase().to_string().as_str() {
        "r" => Figure::Rook,
        "n" => Figure::Knight,
        "b" => Figure::Bishop,
        "q" => Figure::Queen,
        "k" => Figure::King,
        "p" => Figure::Pawn,
        _ => return Err(CouldNotParse::InvalidPiece(character)),
    };

    Ok(Piece {
        figure,
        color: owner(character),
    })
}

fn owner(character: char) -> Player {
    if character.is_uppercase() {
        return Player::White;
    }

    Player::Black
}

fn parse_player(notation: &str) -> Result<Player, CouldNotParse> {
    match notation {
        "b" => Ok(Player::Black),
        "w" => Ok(Player::White),
        _ => Err(CouldNotParse::InvalidPlayerChar(notation.to_owned())),
    }
}

fn parse_castling_rights(_notation: &str) -> Result<CastlingRights, CouldNotParse> {
    // TODO
    Ok(CastlingRights::all())
}

#[cfg(test)]
mod tests {
    use crate::Game;

    use super::*;

    #[test]
    fn it_parses_the_initial_board_state_correctly() {
        let notation = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 1";
        let parsed = parse_state(notation).unwrap();

        assert_eq!(Game::new().start(), parsed);
    }
}
