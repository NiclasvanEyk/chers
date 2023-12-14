use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::cli::prompt;

/// How the connection is initiated.
pub enum Role {
    /// Waits for incoming TCP connections.
    Server,

    /// Initiates a connection with a [Role::Server].
    Client,
}

impl Role {
    pub fn from_string(role: &str) -> Self {
        match role {
            "server" => Self::Server,
            _ => Self::Client,
        }
    }

    pub fn connect(&self) -> Option<TcpStream> {
        match self {
            Self::Server => wait_for_incoming_connections(),
            Self::Client => try_connecting_to_other_client(),
        }
    }
}

// ============================================================================

pub fn wait_for_incoming_connections() -> Option<TcpStream> {
    // We bind port 0 to let the operating system choose a free one for us
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    // And then also tell the user which port we used
    let address = listener.local_addr().unwrap();
    let port = address.port();
    println!("Listening for incoming connections on port {port}...");

    for request in listener.incoming() {
        let Ok(mut stream) = request else {
            println!("A connection failed!");
            continue;
        };

        let address = stream.peer_addr().unwrap();
        let ip = address.ip().to_string();
        let port = address.port();
        let response = prompt(&format!(
            "Got a request from [{ip}:{port}]. Want to accept? [Y/n]"
        ))
        .trim()
        .to_string();

        if response.is_empty() || response.to_lowercase() == "y" {
            let _ = stream.write_all("accept".as_bytes());
            return Some(stream);
        }
    }

    None
}

// ============================================================================

pub fn try_connecting_to_other_client() -> Option<TcpStream> {
    loop {
        let target = prompt("Enter IP and port to connect to:\n")
            .trim()
            .to_string();

        match TcpStream::connect(target) {
            Ok(mut stream) => {
                println!("Waiting for other party to accept the connection request...");
                if other_confirms_connection(&mut stream) {
                    return Some(stream);
                }

                println!("Other party denied the connection request.");
            }
            Err(error) => {
                println!("Something went wrong ({error})! Try again...");
            }
        };
    }
}

fn other_confirms_connection(stream: &mut TcpStream) -> bool {
    let mut buffer = String::new();
    let result = stream.read_to_string(&mut buffer);
    if result.is_err() {
        return false;
    }

    return buffer.trim() == "accept";
}

// =============================================================================
