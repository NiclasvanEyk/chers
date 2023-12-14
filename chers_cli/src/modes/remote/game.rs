use std::net::TcpStream;

use chers::{Coordinate, Game, Move, State};

use crate::{
    rendering::TerminalRenderer,
    terminal::{parse_promotion, prompt_for_coordinate_or_quit, CoordinatePromptResult},
};

use super::coordination::Coordinator;

enum InputState {
    PromptingFrom,
    PromptingTo(Coordinate),
    Execute(Move),
}

pub struct RemoteChersMatch {
    engine: Game,
    renderer: TerminalRenderer,
    coordinator: Coordinator,
    game_state: State,
    input_state: InputState,
}

impl RemoteChersMatch {
    pub fn new(engine: Game, coordinator: Coordinator) -> Self {
        let initial_state = engine.start();

        Self {
            engine,
            renderer: TerminalRenderer {},
            coordinator,
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
        self.renderer.render(&self.game_state.board);

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

                            self.renderer.render(&self.game_state.board);

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
