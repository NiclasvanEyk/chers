use crate::moves::transport::Coordinator;
use chers::{initial_state, move_piece, moves::moves_available, Coordinate, Move, State};

use crate::{
    rendering::TerminalRenderer,
    terminal::{parse_promotion, prompt_for_coordinate_or_quit, CoordinatePromptResult},
};

enum InputState {
    PromptingFrom,
    PromptingTo(Coordinate),
    Execute(Move),
    WaitingForOtherPartyToMove,
}

pub struct RemoteChersMatch {
    renderer: TerminalRenderer,
    coordinator: Coordinator,
    game_state: State,
    input_state: InputState,
}

impl RemoteChersMatch {
    pub fn new(coordinator: Coordinator) -> Self {
        Self {
            renderer: TerminalRenderer {},
            coordinator,
            game_state: initial_state(),
            input_state: InputState::PromptingFrom,
        }
    }

    fn print_possible_moves(&self, from: Coordinate) {
        println!("Possible moves:");

        for possible in moves_available(&self.game_state, from) {
            println!("- {}", possible)
        }
    }

    pub fn run(&mut self) {
        self.renderer.render(&self.game_state.board);

        'game: loop {
            self.input_state = match self.input_state {
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

                InputState::Execute(the_move) => match move_piece(&self.game_state, the_move) {
                    Err(error) => {
                        println!("{:#?}", error);
                        InputState::PromptingTo(the_move.from)
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

                        match self.coordinator.send(&the_move) {
                            Ok(()) => {
                                println!("Waiting for other player to make a move...");
                            }
                            Err(error) => {
                                println!("Something went wrong: {}", error);
                            }
                        }

                        InputState::WaitingForOtherPartyToMove
                    }
                },
                InputState::WaitingForOtherPartyToMove => {
                    // TODO: Actually wait? implement timers?
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    InputState::PromptingFrom
                }
            };
        }
    }
}
