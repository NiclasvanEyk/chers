use std::error::Error;
use std::io::Read;
use std::{io::Write, net::TcpStream};

use crate::Move;

use super::serialization::Converter;

/// Converts moves from and into different string representations.
pub trait Transport {
    /// Send [`a_move`] to the other party
    fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>>;
    /// Wait and block, until the other party has made their move.
    fn receive(&mut self) -> Result<Move, Box<dyn Error>>;
}

/// Juggles transferring moves and updating game state.
pub struct TcpTransport {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Transport for TcpTransport {
    /// Sends a local move to the remote party.
    fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>> {
        let serialized = self.converter.serialize(a_move);

        Ok(self.stream.write_all(serialized.as_bytes())?)
    }

    /// Waits for the remote party to make their move.
    fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        // 128 should be more than enough to serialize a simple move
        let mut buffer: [u8; 128] = [0; 128];
        match self.stream.read_exact(&mut buffer) {
            Ok(it) => it,
            Err(err) => return Err(Box::new(err)),
        };

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}

/// Juggles transferring moves and updating game state.
pub struct Coordinator {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Coordinator {
    pub fn new(stream: TcpStream, converter: Box<dyn Converter>) -> Self {
        Self { stream, converter }
    }

    /// Sends a local move to the remote party.
    pub fn send(&mut self, a_move: &Move) -> Result<(), std::io::Error> {
        let serialized = self.converter.serialize(a_move);

        self.stream.write_all(serialized.as_bytes())
    }

    /// Waits for the remote party to make their move.
    pub fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        // 128 should be more than enough to serialize a simple move
        let mut buffer: [u8; 128] = [0; 128];
        self.stream.read_exact(&mut buffer)?;

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}
