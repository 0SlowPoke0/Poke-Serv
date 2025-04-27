use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, fs, thread};

fn main() {
    println!("Logs from your program will appear here!");

    // Parse command line arguments for directory
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() >= 3 && args[1] == "--directory" {
        args[2].clone()
    } else {
        "./tmp".to_string()
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone directory for use in the thread
                let dir = directory.clone();
                // Spawn a new thread for each connection
                thread::spawn(move || {
                    handle_connection(stream, dir);
                });
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, directory: String) {
    match read_from_stream(&mut stream) {
        Ok(data) => {
            let request_line = data.lines().next().unwrap_or_default();
            let path = request_line.split_whitespace().nth(1).unwrap_or_default();

            let response = handle_endpoint(path, &data, &directory);

            if let Err(e) = stream.write_all(response.as_bytes()) {
                println!("Failed to write response: {}", e);
            }
        }
        Err(e) => {
            println!("Failed to read from stream: {}", e);
        }
    }
}

fn read_from_stream(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    // Set a read timeout to avoid hanging
    stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;

    // Convert bytes read to String
    String::from_utf8(buffer[0..n].to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn user_agent_endpoint(data: &str) -> String {
    let headers: Vec<&str> = data.split("\r\n").collect();
    let mut user_agent = "";

    // Look for the User-Agent header in all headers
    for header in headers {
        if header.starts_with("User-Agent:") {
            user_agent = header.strip_prefix("User-Agent:").unwrap_or("").trim();
            break;
        }
    }

    if user_agent.is_empty() {
        String::from("HTTP/1.1 404 Not Found\r\n\r\n")
    } else {
        let length = user_agent.len();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            length, user_agent
        )
    }
}

fn echo_endpoint(path: &str) -> String {
    // Extract the part after "/echo/"
    let string_to_echo = path.strip_prefix("/echo/").unwrap_or("");
    let length = string_to_echo.len();

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        length, string_to_echo
    )
}

fn files_endpoint(path: &str, directory: &str) -> String {
    // Extract the part after "/files/"
    let file_name = path.strip_prefix("/files/").unwrap_or("");

    // Check if the file name is empty
    if file_name.is_empty() {
        return String::from("HTTP/1.1 404 Not Found\r\n\r\n");
    }

    // Create the full file path
    let file_path = format!("{}/{}", directory, file_name);

    // Use fs::read to read binary content instead of read_to_string
    match fs::read(&file_path) {
        Ok(contents) => {
            // Create header part of the response
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                contents.len()
            );

            // Combine header and content bytes
            let mut response = header.into_bytes();
            response.extend_from_slice(&contents);

            // Convert back to String for consistent interface
            String::from_utf8(response)
                .unwrap_or_else(|_| String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n"))
        }
        Err(_) => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}

fn handle_endpoint(path: &str, data: &str, directory: &str) -> String {
    match path {
        "/" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
        "/user-agent" => user_agent_endpoint(data),
        _ if path.starts_with("/files/") => files_endpoint(path, directory),
        _ if path.starts_with("/echo/") => echo_endpoint(path),
        _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}
