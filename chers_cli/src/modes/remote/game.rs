use chers::{moves::transport::Coordinator, Coordinate, Game, Move, State};

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

    pub async fn run(&mut self) {
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

                InputState::Execute(the_move) => {
                    match self.engine.move_piece(&self.game_state, the_move) {
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

                            match self.coordinator.send(&the_move).await {
                                Ok(()) => {
                                    println!("Waiting for other player to make a move...");
                                }
                                Err(error) => {
                                    println!("Something went wrong: {}", error);
                                }
                            }

                            InputState::WaitingForOtherPartyToMove
                        }
                    }
                }
                InputState::WaitingForOtherPartyToMove => {
                    // TODO: Actually wait? implement timers?
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    InputState::PromptingFrom
                }
            };
        }
    }
}
