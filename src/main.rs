#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::{
    arch::x86_64,
    io::{Read, Write},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let mut buffer = String::from("HTTP/1.1 200 OK\r\n\r\n");

        match stream {
            Ok(mut stream) => {
                let data = match handle_result(read_from_stream(&mut stream)) {
                    Some(data) => data,
                    None => {
                        println!("Failed to read from stream");
                        continue;
                    }
                };

                let path = data.split_whitespace().nth(1).unwrap();
                if path == "/" {
                    buffer = String::from("HTTP/1.1 200 OK\r\n\r\n");
                } else {
                    buffer = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
                }
                stream.write_all(buffer.as_bytes()).unwrap();
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
    let mut buffer = Vec::new();

    loop {
        let mut chunk = vec![0; 1024];
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                chunk.truncate(n);
                buffer.extend_from_slice(&chunk);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Convert to String
    String::from_utf8(buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
