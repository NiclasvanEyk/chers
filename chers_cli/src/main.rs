use std::{env, process::exit};

use chers::Game;

use chers_cli::modes::local::TerminalChersMatch;
use chers_cli::modes::remote::connection::Role;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() > 1 {
        let role = Role::from_string(args[1].as_str());
        let Some(stream) = role.connect() else {
            println!("That did not work. Maybe try again?");
            exit(1);
        };

        let other = stream.peer_addr().unwrap().to_string();
        println!("Successfully connected to {other}!");
        exit(0);
    }

    start_local_game();
    exit(0)
}

fn start_local_game() {
    let engine = Game::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
