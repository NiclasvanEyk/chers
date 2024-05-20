use std::process::exit;

use chers::Game;
use clap::Parser;

use chers::moves::serialization::SimpleMoveConverter;
use chers::moves::transport::Coordinator;
use chers_cli::modes::local::TerminalChersMatch;
use chers_cli::modes::remote::connection::Role;
use chers_cli::modes::remote::game::RemoteChersMatch;

#[derive(Parser)]
#[command(name = "chers")]
#[command(version = "0.0.1-alpha")]
#[command(about = "Play chess on your terminal")]
struct Cli {
    /// When present, starts a remote game.
    ///
    /// If absent, a local game is started, where white and black play on the same terminal.
    role: Option<Role>,

    /// The host to connect to or host from, dependending on the role.
    ///
    /// This is only relevant for remote games. Default: '127.0.0.1' (localhost).
    #[arg(long)]
    host: Option<String>,

    /// The port to connect to or host from, dependending on the role.
    #[arg(long)]
    port: Option<u32>,
}

fn main() {
    let cli = Cli::parse();

    let Some(role) = cli.role else {
        let engine = Game::new();
        let mut ui = TerminalChersMatch::new(engine);

        ui.run();
        exit(0);
    };

    let maybe_stream = match role.connect(cli.host, cli.port) {
        Ok(x) => x,
        Err(error) => {
            println!(
                "Something went wrong while trying to establish a connection: {:?}",
                error
            );
            exit(1);
        }
    };
    let Some(stream) = maybe_stream else {
        println!("That did not work. Maybe try again?");
        exit(1);
    };

    let other = stream.peer_addr().unwrap().to_string();
    println!("Successfully connected to {other}!");

    let engine = Game::new();
    let coordinator = Coordinator::new(stream, Box::new(SimpleMoveConverter::new()));
    let mut ui = RemoteChersMatch::new(engine, coordinator);

    ui.run();
}
