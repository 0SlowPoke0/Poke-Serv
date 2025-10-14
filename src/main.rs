use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;

use anyhow::Result;
use clap::{command, Parser};
use flate2::write::GzEncoder;
use flate2::Compression;

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
                    let mut reader = BufReader::new(&stream);
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line)?;

                    let request_line = request_line.trim();
                    let mut headers = HashMap::new();

                    // Parse headers
                    loop {
                        let mut header_line = String::new();
                        reader.read_line(&mut header_line)?;
                        let trimmed_line = header_line.trim();
                        if trimmed_line.is_empty() {
                            break;
                        }
                        let parts: Vec<&str> = trimmed_line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            headers
                                .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                        }
                    }

                    // Main response logic — all arms return Vec<u8>
                    let response: Vec<u8> = match &request_line[..] {
                        "GET / HTTP/1.1" => b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),

                        p if p.starts_with("GET /user-agent") => {
                            let user_agent = headers.get("User-Agent").unwrap();
                            let body = user_agent.as_bytes();
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                                body.len()
                            );
                            let mut res = header.into_bytes();
                            res.extend_from_slice(body);
                            res
                        }

                        p if p.starts_with("GET /echo/") => {
                            let content = p
                                .strip_prefix("GET /echo/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();

                            if let Some(encoding) = headers.get("Accept-Encoding") {
                                if encoding.split(',').map(|t| t.trim()).any(|t| t == "gzip") {
                                    // gzip compression
                                    let mut encoder =
                                        GzEncoder::new(Vec::new(), Compression::default());
                                    encoder.write_all(content.as_bytes())?;
                                    let compressed_bytes = encoder.finish()?;

                                    let header = format!(
                                        "HTTP/1.1 200 OK\r\n\
                                         Content-Encoding: gzip\r\n\
                                         Content-Type: text/plain\r\n\
                                         Content-Length: {}\r\n\r\n",
                                        compressed_bytes.len()
                                    );
                                    let mut res = header.into_bytes();
                                    res.extend_from_slice(&compressed_bytes);
                                    res
                                } else {
                                    // plain text fallback
                                    let body = content.as_bytes();
                                    let header = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                                        body.len()
                                    );
                                    let mut res = header.into_bytes();
                                    res.extend_from_slice(body);
                                    res
                                }
                            } else {
                                // no encoding header
                                let body = content.as_bytes();
                                let header = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                                    body.len()
                                );
                                let mut res = header.into_bytes();
                                res.extend_from_slice(body);
                                res
                            }
                        }

                        p if p.starts_with("GET /files/") => {
                            let directory_path = args.directory.unwrap_or_else(|| ".".to_string());
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
                                b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec()
                            } else {
                                let mut data = Vec::new();
                                File::open(path)?.read_to_end(&mut data)?;
                                let header = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                    data.len()
                                );
                                let mut res = header.into_bytes();
                                res.extend_from_slice(&data);
                                res
                            }
                        }

                        p if p.starts_with("POST /files/") => {
                            let directory_path = args.directory.unwrap_or_else(|| ".".to_string());
                            let directory = directory_path.trim();
                            let file_name = p
                                .strip_prefix("POST /files/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();

                            let content_length: usize = headers
                                .get("Content-Length")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);

                            let mut body = vec![0u8; content_length];
                            reader.read_exact(&mut body)?;

                            let file_path = format!("{}/{}", directory, file_name);
                            let path = Path::new(&file_path);

                            if let Ok(mut file) = File::create(path) {
                                file.write_all(&body)?;
                                b"HTTP/1.1 201 Created\r\n\r\n".to_vec()
                            } else {
                                b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec()
                            }
                        }

                        _ => b"HTTP/1.1 404 Not Found\r\n\r\n".to_vec(),
                    };

                    stream.write_all(&response)?;
                    Ok(())
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }

    Ok(())
}
