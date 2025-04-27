use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    match read_from_stream(&mut stream) {
        Ok(data) => {
            let request_line = data.lines().next().unwrap_or_default();
            let path = request_line.split_whitespace().nth(1).unwrap_or_default();

            let response = handle_endpoint(path, &data);

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

fn handle_endpoint(path: &str, data: &str) -> String {
    match path {
        "/" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
        "/user-agent" => user_agent_endpoint(data),
        _ if path.starts_with("/files/") => files_endpoint(path),
        _ if path.starts_with("/echo/") => echo_endpoint(path),
        _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}

fn files_endpoint(path: &str) -> String {
    // Extract the part after "/files/"
    let file_name = path.strip_prefix("/files/").unwrap_or("");

    // Check if the file name is empty
    if file_name.is_empty() {
        return String::from("HTTP/1.1 404 Not Found\r\n\r\n");
    }

    // Attempt to read the file
    match fs::read_to_string(format!("./tmp/{}", file_name)) {
        Ok(contents) => {
            // Create a response with the file contents
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                contents.len(),
                contents
            )
        }
        Err(_) => {
            // If the file doesn't exist or can't be read, return a 404 response
            String::from("HTTP/1.1 404 Not Found\r\n\r\n")
        }
    }
}
