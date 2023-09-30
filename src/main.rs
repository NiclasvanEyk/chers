/// A chess implementation.
mod chess;

/// A terminal user interface for playing chess.
mod terminal;

use chess::Engine;
use terminal::TerminalChersMatch;

fn main() {
    let engine = Engine::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
