use std::process::exit;

use chers_cli::modes::local::TerminalChersMatch;
use clap::Parser;

#[derive(Parser)]
#[command(name = "chers")]
#[command(version = "0.0.1-alpha")]
#[command(about = "Play chess on your terminal")]
struct Cli;

fn main() {
    Cli::parse();

    let mut ui = TerminalChersMatch::new();

    ui.run();
    exit(0);
}
