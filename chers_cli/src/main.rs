use std::process::exit;

use clap::Parser;
use chers_cli::modes::local::TerminalChersMatch;

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
