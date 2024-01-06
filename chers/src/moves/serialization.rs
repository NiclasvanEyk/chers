use std::fmt::Display;

use crate::{Coordinate, CoordinateParseError, Move, PromotedFigure, PromotionError};

#[derive(Debug)]
pub enum ConversionError {
    UnknownPromotionPiece(String),
    NoCoordinateProvided { part: String },
    FailedToParseCoordinate { cause: CoordinateParseError },
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnknownPromotionPiece(piece) => {
                write!(f, "Unknown promotion piece '{}'", piece)
            }
            ConversionError::NoCoordinateProvided { part } => {
                write!(f, "No coordinate provided in '{}'", part)
            }
            ConversionError::FailedToParseCoordinate { cause } => {
                write!(f, "Failed to parse Coordinate: '{:?}'", cause)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<CoordinateParseError> for ConversionError {
    fn from(value: CoordinateParseError) -> Self {
        Self::FailedToParseCoordinate { cause: value }
    }
}

/// Converts moves from and into different string representations.
pub trait Converter {
    fn serialize(&self, a_move: &Move) -> String;
    fn deserialize(&self, string: String) -> Result<Move, ConversionError>;
}

/// Serializes just enough information, that two remote parties playing the same
/// game know how to interpret each move.
/// The format is `FROM-TO[-PROMOTION]`, where the last portion is optional.
/// The players local engines are responsible for applying the moves and keep
/// their state local.
pub struct SimpleMoveConverter {}

impl SimpleMoveConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SimpleMoveConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PromotionError> for ConversionError {
    fn from(value: PromotionError) -> Self {
        match value {
            PromotionError::UnknownPiece { piece } => ConversionError::UnknownPromotionPiece(piece),
        }
    }
}

impl Converter for SimpleMoveConverter {
    fn serialize(&self, a_move: &Move) -> String {
        match a_move.promotion {
            None => format!("{}-{}", a_move.from, a_move.to),
            Some(piece) => format!("{}-{}-{}", a_move.from, a_move.to, piece),
        }
    }

    fn deserialize(&self, string: String) -> Result<Move, ConversionError> {
        let mut parts = string.split('-');

        let from = deserialize_coordinate(parts.next(), "from")?;
        let to = deserialize_coordinate(parts.next(), "to")?;
        let promotion = match parts.next() {
            Some(piece_raw) => Some(PromotedFigure::parse(piece_raw)?),
            None => None,
        };

        Ok(Move {
            from,
            to,
            promotion,
        })
    }
}

fn deserialize_coordinate(
    part: Option<&str>,
    description: &str,
) -> Result<Coordinate, ConversionError> {
    let Some(part_raw) = part else {
        return Err(ConversionError::NoCoordinateProvided {
            part: String::from(description),
        });
    };

    Ok(Coordinate::parse(part_raw)?)
}
