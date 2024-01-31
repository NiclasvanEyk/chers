use std::error::Error;
use tokio::net::TcpStream;

use crate::Move;

use super::serialization::Converter;

/// Converts moves from and into different string representations.
pub trait Transport {
    /// Send [`a_move`] to the other party
    async fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>>;

    /// Wait and block, until the other party has made their move.
    async fn receive(&mut self) -> Result<Move, Box<dyn Error>>;
}

/// Juggles transferring moves and updating game state.
pub struct TcpTransport {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Transport for TcpTransport {
    /// Sends a local move to the remote party.
    async fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>> {
        let serialized = self.converter.serialize(a_move);

        self.stream.writable().await;

        match self.stream.try_write(serialized.as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => Err(Box::new(err)),
        }
    }

    /// Waits for the remote party to make their move.
    async fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        self.stream.readable().await;

        // 128 should be more than enough to serialize a simple move
        let mut buffer: [u8; 128] = [0; 128];
        match self.stream.try_read(&mut buffer) {
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
    pub async fn send(&mut self, a_move: &Move) -> Result<(), std::io::Error> {
        let serialized = self.converter.serialize(a_move);

        self.stream.writable().await;
        match self.stream.try_write(serialized.as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Waits for the remote party to make their move.
    pub async fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        self.stream.readable().await;

        // 128 should be more than enough to serialize a simple move
        let mut buffer: [u8; 128] = [0; 128];
        self.stream.try_write(&mut buffer)?;

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}
