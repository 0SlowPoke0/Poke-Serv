use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let buf = b"HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(buf);
                let data = match handle_result(read_from_stream(&mut stream)) {
                    Some(data) => data,
                    None => {
                        println!("Failed to read from stream");
                        continue;
                    }
                };

                let request_line = data.lines().next().unwrap_or_default();
                let path = request_line.split_whitespace().nth(1).unwrap_or_default();
                let response = handle_endpoint(path, data.clone());
                stream.write_all(response.as_bytes());

                if let Err(e) = stream.write_all(response.as_bytes()) {
                    println!("Failed to write response: {}", e);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_result<T, E: std::fmt::Display>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(data) => Some(data),
        Err(_) => {
            println!("An error occurred");
            None
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

fn user_agent_endpoint(data: String) -> String {
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

fn echo_endpoint(data: String) -> String {
    let string_received = data.strip_prefix("/echo/").unwrap_or("");
    let length = string_received.len();

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        length, string_received
    )
}

fn handle_endpoint(path: &str, data: String) -> String {
    match path {
        "/" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
        "/user-agent" => user_agent_endpoint(data),
        _ if path.starts_with("/echo/") => echo_endpoint(path.to_string()),
        _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}
