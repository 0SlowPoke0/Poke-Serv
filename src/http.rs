use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[derive(Debug)]
pub struct Request<'a> {
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub reader: &'a mut BufReader<&'a TcpStream>,
}

impl<'a> Request<'a> {
    /// Parses an incoming stream and constructs a Request object.
    pub fn new(reader: &'a mut BufReader<&'a TcpStream>) -> Result<Self> {
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .context("Failed to read the request line")?;

        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Malformed request line"));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();

        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .context("Failed to read header line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some((key, value)) = trimmed.split_once(':') {
                // Normalize header keys to lowercase for case-insensitive lookup
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        let content_length: usize = headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader
                .read_exact(&mut body)
                .context("Failed to read request body")?;
        }

        Ok(Request {
            method,
            path,
            http_version,
            headers,
            body,
            reader,
        })
    }

    pub fn connection_close_exist(&self) -> bool {
        self.headers
            .get(&"Connection".trim().to_lowercase())
            .is_some_and(|s| *s == "close")
    }
}

/// Represents an outgoing HTTP Response.
/// This struct is used to build the response before sending it.
#[derive(Debug)]
pub struct Response {
    status_code: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    /// Creates a new Response with a given status code.
    pub fn new(status_code: u16, status_text: String) -> Self {
        Response {
            status_code,
            status_text,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Sets the body of the response.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Adds a header to the response.
    pub fn header(mut self, key: &str, value: String) -> Self {
        self.headers.insert(key.to_string(), value);
        self
    }

    pub fn add_connection_close_header(&mut self) {
        self.headers
            .insert("Connection".to_string(), "close".to_string());
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        // Automatically add Content-Length if not present
        if !self.body.is_empty() && !self.headers.contains_key("Content-Length") {
            self.headers
                .insert("Content-Length".to_string(), self.body.len().to_string());
        }

        let mut response_bytes = Vec::new();
        let status_line = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text);
        response_bytes.extend_from_slice(status_line.as_bytes());

        for (key, value) in self.headers {
            let header_line = format!("{}: {}\r\n", key, value);
            response_bytes.extend_from_slice(header_line.as_bytes());
        }

        response_bytes.extend_from_slice(b"\r\n");
        response_bytes.extend_from_slice(&self.body);

        response_bytes
    }
}
