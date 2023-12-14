use chers::{Coordinate, PromotedFigure};

use crate::cli::prompt;

pub fn parse_promotion(input: String) -> Option<PromotedFigure> {
    let Some(promotion) = input.chars().nth(2) else {
        return None;
    };

    match promotion {
        'q' => Some(PromotedFigure::Queen),
        'n' => Some(PromotedFigure::Knight),
        'r' => Some(PromotedFigure::Rook),
        'b' => Some(PromotedFigure::Bishop),
        _ => None,
    }
}

pub enum CoordinatePromptResult {
    Coordinate(Coordinate, String),
    Back,
}

pub fn prompt_for_coordinate_or_quit(question: &str) -> CoordinatePromptResult {
    loop {
        let input = prompt(question);

        match input.trim().to_lowercase().as_str() {
            "b" => return CoordinatePromptResult::Back,
            notation => match Coordinate::algebraic(notation) {
                Ok(coordinate) => {
                    return CoordinatePromptResult::Coordinate(coordinate, notation.to_string());
                }
                Err(err) => println!("{:?}", err),
            },
        };
    }
}
