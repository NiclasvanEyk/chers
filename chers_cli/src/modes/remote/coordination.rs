use std::io::Read;
use std::{io::Write, net::TcpStream};

use chers::moves::serialization::Converter;

use chers::Move;

/// Juggles transferring moves and updating game state.
pub struct Coordinator {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Coordinator {
    /// Sends a local move to the remote party.
    pub fn send(&mut self, a_move: &Move) -> Result<(), std::io::Error> {
        let serialized = self.converter.serialize(a_move);

        self.stream.write_all(serialized.as_bytes())
    }

    /// Waits for the remote party to make their move.
    pub fn receive(&mut self) -> Result<Move, String> {
        let mut buffer: [u8; 128] = [0; 128];
        match self.stream.read(&mut buffer) {
            Ok(it) => it,
            Err(err) => return Err(err.to_string()),
        };

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        return self.converter.deserialize(serialized);
    }
}
