use std::io::Write;

use chers::{Board, Color, Coordinate, Figure, Game, Move, Piece, PromotedFigure, State};

enum InputState {
    PromptingFrom,
    PromptingTo(Coordinate),
    Execute(Move),
}

pub struct TerminalChersMatch {
    engine: Game,
    game_state: State,
    input_state: InputState,
}

impl TerminalChersMatch {
    pub fn new(engine: Game) -> Self {
        let initial_state = engine.start();

        Self {
            engine,
            game_state: initial_state,
            input_state: InputState::PromptingFrom,
        }
    }

    fn print_possible_moves(&self, from: Coordinate) {
        println!("Possible moves:");

        for possible in self.engine.available_moves(&self.game_state, from) {
            println!("- {}", possible)
        }
    }

    pub fn run(&mut self) {
        clear_terminal();
        println!("{}", show_board(&self.game_state.board));

        'game: loop {
            let new_state = match self.input_state {
                InputState::PromptingFrom => {
                    match prompt_for_coordinate_or_quit(&format!(
                        "{:?}'s turn, input from: ",
                        self.game_state.player
                    )) {
                        CoordinatePromptResult::Coordinate(from, _) => {
                            InputState::PromptingTo(from)
                        }
                        CoordinatePromptResult::Back => InputState::PromptingFrom,
                    }
                }

                InputState::PromptingTo(from) => {
                    self.print_possible_moves(from);
                    match prompt_for_coordinate_or_quit(&format!(
                        "{:?}'s turn, input to: ",
                        self.game_state.player
                    )) {
                        CoordinatePromptResult::Coordinate(to, input) => {
                            InputState::Execute(Move {
                                from,
                                to,
                                promotion: parse_promotion(input),
                            })
                        }
                        CoordinatePromptResult::Back => InputState::PromptingFrom,
                    }
                }

                InputState::Execute(r#move) => {
                    match self.engine.move_piece(&self.game_state, r#move) {
                        Err(error) => {
                            println!("{:#?}", error);
                            InputState::PromptingTo(r#move.from)
                        }
                        Ok((new_state, events)) => {
                            let current_player = self.game_state.player;
                            self.game_state = new_state;
                            clear_terminal();
                            println!("{}", show_board(&self.game_state.board));

                            for event in events {
                                println!("{:?}", event);
                                if let chers::Event::Mate = event {
                                    println!("{:?} wins!", current_player);
                                    break 'game;
                                }
                            }

                            InputState::PromptingFrom
                        }
                    }
                }
            };

            self.input_state = new_state;
        }
    }
}

fn parse_promotion(input: String) -> Option<PromotedFigure> {
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

fn prompt(question: &str) -> String {
    let mut x = String::new();
    print!("{}", question);
    std::io::stdout().flush().expect("Could not flush stdout");

    std::io::stdin()
        .read_line(&mut x)
        .expect("Could not read input");

    x
}

enum CoordinatePromptResult {
    Coordinate(Coordinate, String),
    Back,
}

fn prompt_for_coordinate_or_quit(question: &str) -> CoordinatePromptResult {
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

    format!("{background}{}{} \x1b[0m", foreground, piece_str)
}

pub fn show_board(board: &Board) -> String {
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
                + format!(" {}", 8 - row_index).as_str()
        })
        .collect::<Vec<String>>()
        .join("\n")
        + "\na b c d e f g h\n";
}

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
