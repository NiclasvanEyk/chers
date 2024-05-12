use chers::{moves::transport::Transport, Coordinate, Game, Move, State};

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

pub struct RemoteChersMatch<T>
where
    T: Transport,
{
    engine: Game,
    renderer: TerminalRenderer,
    transport: T,
    game_state: State,
    input_state: InputState,
}

impl<T> RemoteChersMatch<T>
where
    T: Transport,
{
    pub fn new(engine: Game, transport: T) -> Self {
        let initial_state = engine.start();

        Self {
            engine,
            renderer: TerminalRenderer {},
            transport,
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

                            match self.transport.send(&the_move).await {
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

#[cfg(test)]
mod tests {
    use chers::moves::transport::InMemoryTransport;
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    #[tokio::test]
    async fn test_remote_games_can_be_run() {
        println!("Connection successful!");

        // let game_server = Game::new();
        // let converter_server = Box::new(SimpleMoveConverter::new());
        // let coordinator_server = Coordinator::new(socket_server, converter_server);
        //
        // let game_client = Game::new();
    }
}
