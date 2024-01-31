use tokio::net::TcpListener;
use tokio::net::TcpStream;

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

    pub async fn connect(
        &self,
        host: Option<String>,
        port: Option<u32>,
    ) -> std::io::Result<Option<TcpStream>> {
        match self {
            Self::Server => wait_for_incoming_connections(host, port).await,
            Self::Client => try_connecting_to_other_client(host, port).await,
        }
    }
}

// ============================================================================
pub async fn wait_for_incoming_connections(
    host: Option<String>,
    port: Option<u32>,
) -> std::io::Result<Option<TcpStream>> {
    // We bind port 0 to let the operating system choose a free one for us
    let listener = TcpListener::bind(format!(
        "{}:{}",
        host.unwrap_or(String::from("127.0.0.1")),
        port.unwrap_or(0),
    ))
    .await?;

    // And then also tell the user which port we used
    let address = listener.local_addr()?;
    let port = address.port();
    println!("Listening for incoming connections on port {port}...");

    loop {
        let (stream, address) = listener.accept().await?;
        let ip = address.ip().to_string();
        let port = address.port();
        let response = prompt(&format!(
            "Got a request from [{ip}:{port}]. Want to accept? [Y/n]"
        ))
        .trim()
        .to_string();

        if response.is_empty() || response.to_lowercase() == "y" {
            stream.try_write("accept".as_bytes())?;
            return Ok(Some(stream));
        }
    }
}

// ============================================================================

pub async fn try_connecting_to_other_client(
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

        match TcpStream::connect(target).await {
            Ok(mut stream) => {
                println!("Waiting for other party to accept the connection request...");
                if other_confirms_connection(&mut stream).await {
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

async fn other_confirms_connection(stream: &mut TcpStream) -> bool {
    loop {
        // Wait for the socket to be readable
        stream.readable().await;

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut buf = [0; 4096];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut buf) {
            Ok(0) => {
                println!("Nothing was read");
                return false;
            }
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf);
                println!("read {} bytes ('{}')", n, received);
                return if received == "accept" {
                    true
                } else {
                    println!("Did not accept");
                    false
                };
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                println!(
                    "An error occurred while trying to read from socket: {}",
                    err
                );
                return false;
            }
        }
    }
}

// =============================================================================
