pub mod cli;
mod remote_play;
/// A terminal user interface for playing chess.
mod terminal;

use std::env;

use chers::Game;
use remote_play::{try_connecting_to_other_client, wait_for_incoming_connections};
use terminal::TerminalChersMatch;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() > 1 {
        let result = match (args[1].as_str()) {
            "server" => wait_for_incoming_connections(),
            _ => try_connecting_to_other_client(),
        };
    }

    let engine = Game::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
