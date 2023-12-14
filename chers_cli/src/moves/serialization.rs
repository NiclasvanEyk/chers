use chers::{moves::serialization::Converter, Coordinate, Move, PromotedFigure};

/// Serializes just enough information, that two remote parties playing the same
/// game know how to interpret each move.
pub struct Remote {}

impl Converter for Remote {
    fn serialize(&self, a_move: &Move) -> String {
        match a_move.promotion {
            None => format!("{}-{}", a_move.from, a_move.to),
            Some(piece) => format!("{}-{}-{}", a_move.from, a_move.to, piece),
        }
    }

    fn deserialize(&self, string: String) -> Result<Move, String> {
        let mut parts = string.split("-");

        let from = deserialize_coordinate(parts.next(), "from")?;
        let to = deserialize_coordinate(parts.next(), "to")?;
        let promotion = match parts.next() {
            Some(piece_raw) => Some(PromotedFigure::parse(piece_raw)?),
            None => None,
        };

        return Ok(Move {
            from,
            to,
            promotion,
        });
    }
}

fn deserialize_coordinate(part: Option<&str>, description: &str) -> Result<Coordinate, String> {
    let Some(part_raw) = part else {
        return Err(String::from(format!(
            "No '{description}' coordinate specified!"
        )));
    };

    return match Coordinate::parse(part_raw) {
        Ok(coordinate) => Ok(coordinate),
        Err(message) => Err(message),
    };
}
