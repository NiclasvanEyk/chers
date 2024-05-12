use std::{cell::RefCell, error::Error};
use tokio::net::TcpStream;

use crate::Move;

use super::serialization::Converter;

/// Converts moves from and into different string representations.
pub trait Transport {
    /// Send [`a_move`] to the other party
    async fn send(&self, a_move: &Move) -> Result<(), Box<dyn Error>>;

    /// Wait and block, until the other party has made their move.
    async fn receive(&self) -> Result<Move, Box<dyn Error>>;
}

// ============================================================================

/// Juggles transferring moves and updating game state.
pub struct TcpTransport {
    stream: RefCell<TcpStream>,
    converter: Box<dyn Converter>,
}

impl TcpTransport {
    pub fn new(stream: TcpStream, converter: Box<dyn Converter>) -> Self {
        Self {
            stream: RefCell::new(stream),
            converter,
        }
    }
}

impl Transport for TcpTransport {
    /// Sends a local move to the remote party.
    async fn send(&self, a_move: &Move) -> Result<(), Box<dyn Error>> {
        let serialized = self.converter.serialize(a_move);

        let _ = self.stream.borrow().writable().await;

        match self.stream.borrow().try_write(serialized.as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => Err(Box::new(err)),
        }
    }

    /// Waits for the remote party to make their move.
    async fn receive(&self) -> Result<Move, Box<dyn Error>> {
        let _ = self.stream.borrow().readable().await;

        // 128 should be more than enough to serialize a simple move
        let mut buffer: [u8; 128] = [0; 128];
        match self.stream.borrow().try_read(&mut buffer) {
            Ok(it) => it,
            Err(err) => return Err(Box::new(err)),
        };

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}

// ============================================================================

#[derive(Default)]
struct MaybeMove {
    pub value: Option<Move>,
}

#[derive(Default)]
struct Tunnel {
    inner: RefCell<MaybeMove>,
}

impl Tunnel {
    pub fn read(&self) -> Option<Move> {
        self.inner.borrow().value
    }

    pub fn consume(&self) -> Option<Move> {
        let value = self.read().clone();
        self.inner.borrow_mut().value = None;

        value
    }

    pub fn write(&self, a_move: Move) {
        self.inner.borrow_mut().value = Some(a_move);
    }
}

/// An in-memory transport intended for testing
pub struct InMemoryTransport<'a> {
    pub tunnel: &'a Tunnel,
}

impl<'a> InMemoryTransport<'a> {
    pub fn split() -> (Self, Self) {
        let tunnel = Tunnel::default();

        let a = InMemoryTransport { tunnel: &tunnel };
        let b = InMemoryTransport { tunnel: &tunnel };

        return (a, b);
    }
}

impl<'a> Transport for InMemoryTransport<'a> {
    async fn send(&self, a_move: &Move) -> Result<(), Box<dyn Error>> {
        self.tunnel.write(*a_move);

        Ok(())
    }

    async fn receive(&self) -> Result<Move, Box<dyn Error>> {
        match self.tunnel.consume() {
            Some(a_move) => Ok(a_move),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Nothing to receive!",
            ))),
        }
    }
}
