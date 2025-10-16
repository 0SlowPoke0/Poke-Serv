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
    #[arg(short, long)]
    directory: Option<String>,

    #[arg(short, long = "Accept-Encoding")]
    encoding: Option<String>,
}

fn build_response(
    status_line: &str,
    headers: &[(&str, String)],
    body: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let mut header_bytes = Vec::new();
    let mut header_string = format!("{}\r\n", status_line);
    for (key, value) in headers {
        header_string.push_str(&format!("{}: {}\r\n", key, value));
    }
    header_string.push_str("\r\n");
    header_bytes.extend_from_slice(header_string.as_bytes());
    (header_bytes, body.to_vec())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || -> Result<()> {
                    loop {
                        let args = Args::parse();
                        let mut reader = BufReader::new(&stream);
                        let mut request_line = String::new();
                        reader.read_line(&mut request_line)?;
                        let request_line = request_line.trim();

                        let mut headers = HashMap::new();
                        loop {
                            let mut line = String::new();
                            reader.read_line(&mut line)?;
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                break;
                            }
                            if let Some((k, v)) = trimmed.split_once(':') {
                                headers.insert(k.trim().to_string(), v.trim().to_string());
                            }
                        }

                        // Prepare response (status_line + headers, body)
                        let (header_bytes, body_bytes) = if request_line == "GET / HTTP/1.1" {
                            build_response("HTTP/1.1 200 OK", &[], b"")
                        } else if request_line.starts_with("GET /user-agent") {
                            let user_agent = headers.get("User-Agent").unwrap();
                            build_response(
                                "HTTP/1.1 200 OK",
                                &[
                                    ("Content-Type", "text/plain".into()),
                                    ("Content-Length", user_agent.len().to_string()),
                                ],
                                user_agent.as_bytes(),
                            )
                        } else if request_line.starts_with("GET /echo/") {
                            let content = request_line
                                .strip_prefix("GET /echo/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();

                            if headers
                                .get("Accept-Encoding")
                                .map_or(false, |v| v.split(',').any(|e| e.trim() == "gzip"))
                            {
                                let mut encoder =
                                    GzEncoder::new(Vec::new(), Compression::default());
                                encoder.write_all(content.as_bytes())?;
                                let compressed = encoder.finish()?;
                                build_response(
                                    "HTTP/1.1 200 OK",
                                    &[
                                        ("Content-Type", "text/plain".into()),
                                        ("Content-Encoding", "gzip".into()),
                                        ("Content-Length", compressed.len().to_string()),
                                    ],
                                    &compressed,
                                )
                            } else {
                                build_response(
                                    "HTTP/1.1 200 OK",
                                    &[
                                        ("Content-Type", "text/plain".into()),
                                        ("Content-Length", content.len().to_string()),
                                    ],
                                    content.as_bytes(),
                                )
                            }
                        } else if request_line.starts_with("GET /files/") {
                            let directory = args.directory.clone().unwrap_or_else(|| ".".into());
                            let file_name = request_line
                                .strip_prefix("GET /files/")
                                .unwrap()
                                .strip_suffix("HTTP/1.1")
                                .unwrap()
                                .trim();
                            let path_string = format!("{}/{}", directory, file_name);
                            let path = Path::new(&path_string);
                            if path.exists() {
                                let mut data = Vec::new();
                                File::open(path)?.read_to_end(&mut data)?;
                                build_response(
                                    "HTTP/1.1 200 OK",
                                    &[
                                        ("Content-Type", "application/octet-stream".into()),
                                        ("Content-Length", data.len().to_string()),
                                    ],
                                    &data,
                                )
                            } else {
                                build_response("HTTP/1.1 404 Not Found", &[], b"")
                            }
                        } else if request_line.starts_with("POST /files/") {
                            let directory = args.directory.clone().unwrap_or_else(|| ".".into());
                            let file_name = request_line
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

                            let path_string = format!("{}/{}", directory, file_name);
                            let path = Path::new(&path_string);
                            if let Ok(mut f) = File::create(path) {
                                f.write_all(&body)?;
                                build_response("HTTP/1.1 201 Created", &[], b"")
                            } else {
                                build_response("HTTP/1.1 404 Not Found", &[], b"")
                            }
                        } else {
                            build_response("HTTP/1.1 404 Not Found", &[], b"")
                        };

                        // You can now add common headers here if needed
                        // For example:
                        // let common_headers = b"Server: MyRustServer\r\n";
                        // stream.write_all(common_headers)?;

                        // After building (header_bytes, body_bytes)
                        let mut header_bytes = header_bytes; // make mutable to modify

                        if headers.get("Connection").is_some_and(|t| t == "close") {
                            // Add Connection: close header
                            let mut new_header = Vec::new();
                            let header_str = String::from_utf8_lossy(&header_bytes);
                            // Append Connection: close before final CRLF
                            let header_with_close =
                                header_str.replace("\r\n\r\n", "\r\nConnection: close\r\n\r\n");
                            new_header.extend_from_slice(header_with_close.as_bytes());
                            header_bytes = new_header;

                            // Send response
                            stream.write_all(&header_bytes)?;
                            stream.write_all(&body_bytes)?;
                            // Then exit the loop/thread to close the connection
                            return Ok(());
                        }

                        stream.write_all(&header_bytes)?;
                        stream.write_all(&body_bytes)?;
                    }
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }

    Ok(())
}
