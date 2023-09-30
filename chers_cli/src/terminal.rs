use std::io::Write;

use chers::{Board, Color, Coordinate, CoordinateParserError, Engine, Figure, Move, Piece, State};

pub struct TerminalChersMatch {
    engine: Engine,
    state: State,
}

impl TerminalChersMatch {
    pub fn new(engine: Engine) -> Self {
        let initial_state = engine.start();

        Self {
            engine,
            state: initial_state,
        }
    }

    pub fn run(&mut self) {
        clear_terminal();
        println!("{}", show_board(self.state.board));

        loop {
            let r#move = prompt_for_move(&self.state.player);

            match self.engine.move_piece(&self.state, r#move) {
                Err(error) => {
                    println!("{:#?}", error);
                }
                Ok((new_state, events)) => {
                    self.state = new_state;
                    clear_terminal();
                    println!("{}", show_board(self.state.board));

                    for event in events {
                        println!("{:?}", event);
                    }
                }
            };
        }
    }
}

fn color_for_coordinate(coordinate: Coordinate) -> Color {
    // Even rows have even numbers black, odd rows have odd ones
    if coordinate.y % 2 == coordinate.x % 2 {
        Color::White
    } else {
        Color::Black
    }
}

fn piece_to_string(piece: &Piece) -> String {
    match piece.color {
        Color::White => match piece.figure {
            Figure::King => "♔",
            Figure::Queen => "♕",
            Figure::Rook => "♜",
            Figure::Bishop => "♗",
            Figure::Knight => "♞",
            Figure::Pawn => "♙",
        },
        Color::Black => match piece.figure {
            Figure::King => "♚",
            Figure::Queen => "♛",
            Figure::Rook => "♜",
            Figure::Bishop => "♝",
            Figure::Knight => "♞",
            Figure::Pawn => "♙",
        },
    }
    .to_string()
}

#[derive(Debug)]
enum ReadMoveError {
    InvalidFrom(CoordinateParserError),
    InvalidTo(CoordinateParserError),
}

fn parse_move(input: &str) -> Result<Move, ReadMoveError> {
    let words: Vec<&str> = input.trim().split(' ').take(2).collect();

    let from = match words.first() {
        Some(word) => match Coordinate::algebraic(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidFrom(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    let to = match words.get(1) {
        Some(word) => match Coordinate::algebraic(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidTo(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    Ok(Move { from, to })
}

fn prompt_for_move(player: &Color) -> Move {
    loop {
        let mut x = String::new();
        print!("{:?}'s turn: ", player);
        std::io::stdout().flush().expect("Could not flush stdout");

        std::io::stdin()
            .read_line(&mut x)
            .expect("Could not read input");

        match parse_move(&x) {
            Ok(from_to) => return from_to,
            Err(err) => println!("{:?}", err),
        };
    }
}

fn to_terminal_string(coordinate: Coordinate, piece: Option<Piece>) -> String {
    let background = match color_for_coordinate(coordinate) {
        Color::White => "\x1b[48;5;216m",
        Color::Black => "\x1b[48;5;173m",
    };
    let foreground = match piece {
        Some(p) => match p.color {
            Color::White => "\x1b[38;2;255;255;255m",
            Color::Black => "\x1b[38;2;0;0;0m",
        },
        None => "",
    };

    let piece_str = match piece {
        Some(p) => piece_to_string(&p),
        None => " ".to_string(),
    };

    format!("{}{}{} \x1b[0m", background, foreground, piece_str)
}

pub fn show_board(board: Board) -> String {
    return board
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .map(|(cell_index, piece)| {
                    let coordinate = Coordinate {
                        x: 8 - cell_index,
                        y: 8 - row_index,
                    };

                    to_terminal_string(coordinate, *piece)
                })
                .collect::<Vec<String>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join("\n");
}

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}