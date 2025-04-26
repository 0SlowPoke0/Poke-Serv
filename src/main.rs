use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let data = match handle_result(read_from_stream(&mut stream)) {
                    Some(data) => data,
                    None => {
                        println!("Failed to read from stream");
                        continue;
                    }
                };

                let request_line = data.lines().next().unwrap_or_default();
                let path = request_line.split_whitespace().nth(1).unwrap_or_default();

                let response = if path == "/" {
                    String::from("HTTP/1.1 200 OK\r\n\r\n")
                } else if path.starts_with("/echo/") {
                    // Extract the content after "/echo/"
                    let string_received = path.strip_prefix("/echo/").unwrap_or("");
                    let length = string_received.len();

                    // Use format! to properly interpolate the variables
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        length,
                        string_received
                    )
                } else if path.starts_with("/user-agent") {
                    print!("hi");
                    let user_agent = request_line.split("\r\n").nth(2).unwrap_or_default();
                    if user_agent.is_empty() {
                        String::from("HTTP/1.1 404 Not Found\r\n\r\n")
                    } else {
                        let user_agent_details =
                            user_agent.strip_prefix("User-Agent: ").unwrap_or_default();
                        let length = user_agent_details.len() - 1;
                        format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                length,
                                user_agent_details
                            )
                    }
                } else {
                    String::from("HTTP/1.1 404 Not Found\r\n\r\n")
                };

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
