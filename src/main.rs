use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut request: Vec<u8> = Vec::new();
                let mut buffer = String::from("HTTP/1.1 200 OK\r\n\r\n");
                match stream.read_exact(&mut request) {
                    Ok(_) => {
                        let response_string = String::from_utf8(request).unwrap();
                        let path = response_string
                            .lines()
                            .next()
                            .unwrap()
                            .split_whitespace()
                            .collect::<Vec<_>>()[1];
                        if path == "" {
                            buffer = String::from(
                                "HTTP/1.1 200 OK\r\n\r\n
",
                            );
                        } else {
                            buffer = String::from(
                                "HTTP/1.1 404 Not Found\r\n\r\n
",
                            );
                        }
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                };
                stream.write_all(buffer.as_bytes());
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
