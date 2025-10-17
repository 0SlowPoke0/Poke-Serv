use anyhow::{Context, Result};
use clap::Parser;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

mod cli;
mod handler;
mod http;

use cli::Args;
use handler::handle_request;
use http::Request;

fn main() -> Result<()> {
    // Parse arguments ONCE at startup
    let args = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:4221").context("Failed to bind to port 4221")?;
    println!("Server listening on 127.0.0.1:4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone the args needed for the thread
                let args_clone = args.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, &args_clone) {
                        eprintln!("Error handling connection: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}

/// Manages the lifecycle of a single TCP connection.
fn handle_connection(mut stream: TcpStream, args: &Args) -> Result<()> {
    loop {
        let mut reader = BufReader::new(&stream);
        let request = Request::new(&mut reader).context("Failed to parse request")?;
        let connection_close_header = request.connection_close_exist();

        let mut response = handle_request(request, args).context("Failed to handle request")?;

        if connection_close_header {
            response.add_connection_close_header();
        }

        stream
            .write_all(&response.into_bytes())
            .context("Failed to write response to stream")?;
        stream.flush().context("Failed to flush stream")?;

        if connection_close_header {
            break;
        }
    }
    Ok(())
}
