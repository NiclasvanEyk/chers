/// A terminal user interface for playing chess.
mod terminal;

use chers::Engine;
use terminal::TerminalChersMatch;

fn main() {
    let engine = Engine::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
