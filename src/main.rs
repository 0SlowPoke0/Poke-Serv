use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;

use anyhow::Result;
use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to serve
    #[arg(short, long)]
    directory: Option<String>,

    #[arg(short, long = "Accept-Encoding")]
    encoding: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    let mut count = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("{}", count);
                count += 1;

                thread::spawn(move || -> Result<(), anyhow::Error> {
                    let args = Args::parse();
                    let mut reader = BufReader::new(&stream); // Make reader mutable
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line)?;

                    let request_line = request_line.trim();
                    let mut headers = HashMap::new();
                    loop {
                        let mut header_line = String::new();
                        reader.read_line(&mut header_line)?;
                        let trimmed_line = header_line.trim();
                        if trimmed_line.is_empty() {
                            break; // This is the \r\n\r\n, end of headers
                        }
                        let parts: Vec<&str> = trimmed_line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            headers
                                .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                        }
                    }

                    let response = match &request_line[..] {
                        "GET / HTTP/1.1" => format!("HTTP/1.1 200 OK\r\n\r\n"),
                        p if p.starts_with("GET /user-agent") => {
                            let user_agent = headers.get("User-Agent").unwrap();
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",user_agent.len(),user_agent)
                        }

                        p if p.starts_with("GET /echo/") => {
                            let content = p
                                .strip_prefix("GET /echo/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap();
                            let content_len = content.trim().len();

                            if let Some(encoding) = headers.get("Accept-Encoding") {
                                if encoding == "invalid-encoding" {
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                        content_len, content
                                    )
                                } else {
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                        content_len, content
                                    )
                                }
                            } else {
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                    content_len, content
                                )
                            }
                        }

                        p if p.starts_with("GET /files/") => {
                            let directory_path = args.directory.unwrap();
                            let directory = directory_path.trim();
                            let file_name = p
                                .strip_prefix("GET /files/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();

                            let file_path = format!("{}/{}", directory, file_name);
                            let path = Path::new(&file_path);

                            if !path.exists() {
                                format!("HTTP/1.1 404 Not Found\r\n\r\n")
                            } else {
                                let mut content = BufReader::new(File::open(path).unwrap());
                                let mut data_string = String::new();
                                let _ = content.read_to_string(&mut data_string).unwrap();
                                let content_len = data_string.trim().len();
                                format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {content_len}\r\n\r\n{data_string}")
                            }
                        }

                        p if p.starts_with("POST /files") => {
                            let directory_path = args.directory.unwrap();
                            let directory = directory_path.trim();
                            let file_name = p
                                .strip_prefix("POST /files/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();

                            println!("{}", file_name);

                            let content_length: usize = headers
                                .get("Content-Length")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);

                            // 1. Read from the BufReader, not the underlying stream
                            let mut body = vec![0u8; content_length];
                            reader.read_exact(&mut body)?;
                            // let mut body = vec![0u8; content_length];
                            // reader.get_ref().read_exact(&mut body)?;

                            let body_str = String::from_utf8_lossy(&body);

                            let file_path = format!("{}/{}", directory, file_name);
                            let path = Path::new(&file_path);

                            println!("am i reaching here");
                            println!("{:?}", path);

                            // std::fs::create_dir_all(directory_path)?;
                            if let Ok(mut file) = File::create(path) {
                                let _ = file.write_all(body_str.as_bytes());
                                String::from("HTTP/1.1 201 Created\r\n\r\n")
                            } else {
                                String::from("HTTP/1.1 404 Not Found\r\n\r\n")
                            }
                        }

                        _ => format!("HTTP/1.1 404 Not Found\r\n\r\n"),
                    };

                    stream.write_all(response.as_bytes())?;
                    Ok(())
                });
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}
