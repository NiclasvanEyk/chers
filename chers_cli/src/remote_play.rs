use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::cli::prompt;

// ============================================================================
// Connection negotiation
//
// You either wait for another client to connect to your port, or try to
// connect to another waiting client.
// ============================================================================

pub fn wait_for_incoming_connections() -> Option<TcpStream> {
    // We bind port 0 to let the operating system choose a free one for us
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();

    // And then also tell the user which port we used
    let address = listener.local_addr().unwrap();
    let port = address.port();
    println!("Listening for incoming connections on port {port}...");

    for request in listener.incoming() {
        let Ok(stream) = request else {
            println!("A connection failed!");
            continue;
        };

        let address = stream.peer_addr().unwrap();
        let ip = address.ip().to_string();
        let port = address.port();
        let response = prompt(&format!(
            "Got a request from [{ip}:{port}]. Want to accept? [Y/n]"
        ));
        if response.is_empty() || response.to_lowercase() == "y" {
            return Some(stream);
        }
    }

    None
}

pub fn try_connecting_to_other_client() -> Option<TcpStream> {
    loop {
        let target = prompt("Enter IP and port to connect to:\n");
        match TcpStream::connect(target) {
            Ok(stream) => stream,
            Err(error) => {
                println!("Something went wrong ({error})! Try again...");
                continue;
            }
        };
    }
}
//
// fn example() {
//     fn main() -> std::io::Result<()> {
//         let mut stream = TcpStream::connect("127.0.0.1:34254")?;
//
//         stream.write(&[1])?;
//         stream.read(&mut [0; 128])?;
//         Ok(())
//     }
// }

// ============================================================================
