use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

use clap::ValueEnum;

use crate::cli::prompt;

/// How the connection is initiated.
#[derive(ValueEnum, Clone, Debug)]
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

    pub fn connect(
        &self,
        host: Option<String>,
        port: Option<u32>,
    ) -> std::io::Result<Option<TcpStream>> {
        match self {
            Self::Server => wait_for_incoming_connections(host, port),
            Self::Client => try_connecting_to_other_client(host, port),
        }
    }
}

// ============================================================================
pub fn wait_for_incoming_connections(
    host: Option<String>,
    port: Option<u32>,
) -> std::io::Result<Option<TcpStream>> {
    // We bind port 0 to let the operating system choose a free one for us
    let listener = TcpListener::bind(format!(
        "{}:{}",
        host.unwrap_or(String::from("127.0.0.1")),
        port.unwrap_or(0),
    ))?;

    // And then also tell the user which port we used
    let address = listener.local_addr()?;
    let port = address.port();
    println!("Listening for incoming connections on port {port}...");

    for request in listener.incoming() {
        let Ok(mut stream) = request else {
            println!("A connection failed!");
            continue;
        };

        let address = stream.peer_addr()?;
        let ip = address.ip().to_string();
        let port = address.port();
        let response = prompt(&format!(
            "Got a request from [{ip}:{port}]. Want to accept? [Y/n]"
        ))
        .trim()
        .to_string();

        if response.is_empty() || response.to_lowercase() == "y" {
            stream.write_all("accept".as_bytes())?;
            return Ok(Some(stream));
        }
    }

    Ok(None)
}

// ============================================================================

pub fn try_connecting_to_other_client(
    host: Option<String>,
    port: Option<u32>,
) -> std::io::Result<Option<TcpStream>> {
    loop {
        let real_host = host
            .clone()
            .unwrap_or_else(|| prompt("Enter host to connect to:\n").trim().to_string());
        let real_port = port
            .map(|numeric_port| numeric_port.to_string())
            .unwrap_or_else(|| prompt("Enter port to connect to:\n").trim().to_string());
        let target = format!("{}:{}", real_host, real_port);

        match TcpStream::connect(target) {
            Ok(mut stream) => {
                println!("Waiting for other party to accept the connection request...");
                if other_confirms_connection(&mut stream) {
                    return Ok(Some(stream));
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
