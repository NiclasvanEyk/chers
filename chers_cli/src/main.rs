pub mod cli;

/// Utlities allowing players to compete over the internet.
mod remote_play;

/// A terminal user interface for playing chess.
mod terminal;

use std::{env, process::exit};

use chers::Game;
use remote_play::Role;
use remote_play::{try_connecting_to_other_client, wait_for_incoming_connections};
use terminal::TerminalChersMatch;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() > 1 {
        let role = Role::from_string(args[1].as_str());
        start_remote_game(role);
        exit(0);
    }

    start_local_game();
    exit(0)
}

fn start_remote_game(role: Role) {
    let Some(stream) = (match role {
        Role::Server => wait_for_incoming_connections(),
        Role::Client => try_connecting_to_other_client(),
    }) else {
        println!("That did not work. Maybe try again?");
        exit(1);
    };

    let other = stream.peer_addr().unwrap().to_string();
    println!("Successfully connected to {other}!");
}

fn start_local_game() {
    let engine = Game::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
