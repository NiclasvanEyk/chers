/// A terminal user interface for playing chess.
mod terminal;

use chers::Game;
use terminal::TerminalChersMatch;

fn main() {
    let engine = Game::new();
    let mut ui = TerminalChersMatch::new(engine);

    ui.run();
}
