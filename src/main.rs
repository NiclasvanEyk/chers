mod chess;
mod terminal;

use chess::Engine;
use terminal::TerminalChersMatch;

fn main() {
    let engine = Engine::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
