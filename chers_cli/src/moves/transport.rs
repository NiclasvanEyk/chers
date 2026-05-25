use std::error::Error;
use std::io::Read;
use std::{io::Write, net::TcpStream};

use chers::Move;

use super::serialization::Converter;

pub trait Transport {
    fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>>;
    fn receive(&mut self) -> Result<Move, Box<dyn Error>>;
}

pub struct TcpTransport {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Transport for TcpTransport {
    fn send(&mut self, a_move: &Move) -> Result<(), Box<dyn Error>> {
        let serialized = self.converter.serialize(a_move);

        Ok(self.stream.write_all(serialized.as_bytes())?)
    }

    fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        let mut buffer: [u8; 128] = [0; 128];
        match self.stream.read_exact(&mut buffer) {
            Ok(it) => it,
            Err(err) => return Err(Box::new(err)),
        };

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}

pub struct Coordinator {
    stream: TcpStream,
    converter: Box<dyn Converter>,
}

impl Coordinator {
    pub fn new(stream: TcpStream, converter: Box<dyn Converter>) -> Self {
        Self { stream, converter }
    }

    pub fn send(&mut self, a_move: &Move) -> Result<(), std::io::Error> {
        let serialized = self.converter.serialize(a_move);

        self.stream.write_all(serialized.as_bytes())
    }

    pub fn receive(&mut self) -> Result<Move, Box<dyn Error>> {
        let mut buffer: [u8; 128] = [0; 128];
        self.stream.read_exact(&mut buffer)?;

        let serialized = String::from_utf8_lossy(&buffer).to_string();

        Ok(self.converter.deserialize(serialized)?)
    }
}
