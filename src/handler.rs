use crate::cli::Args;
use crate::http::{Request, Response};
use anyhow::{Context, Result};
use flate2::{write::GzEncoder, Compression};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn handle_request(request: Request, args: &Args) -> Result<Response> {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => Ok(Response::new(200, "OK".to_string())),

        ("GET", "/user-agent") => {
            let user_agent = request
                .headers
                .get("user-agent")
                .cloned()
                .unwrap_or_default();
            Ok(Response::new(200, "OK".to_string())
                .header("Content-Type", "text/plain".to_string())
                .body(user_agent.into_bytes()))
        }

        ("GET", path) if path.starts_with("/echo/") => {
            let content = path.strip_prefix("/echo/").unwrap_or("").as_bytes();

            // Check for gzip compression
            if request
                .headers
                .get("accept-encoding")
                .map_or(false, |v| v.contains("gzip"))
            {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(content)
                    .context("Failed to write to gzip encoder")?;
                let compressed_body = encoder.finish().context("Failed to finish gzip encoding")?;

                Ok(Response::new(200, "OK".to_string())
                    .header("Content-Type", "text/plain".to_string())
                    .header("Content-Encoding", "gzip".to_string())
                    .body(compressed_body))
            } else {
                Ok(Response::new(200, "OK".to_string())
                    .header("Content-Type", "text/plain".to_string())
                    .body(content.to_vec()))
            }
        }

        ("GET", path) if path.starts_with("/files/") => {
            let directory = args.directory.clone().unwrap_or_else(|| ".".into());
            let file_name = path.strip_prefix("/files/").unwrap_or("");
            let file_path = Path::new(&directory).join(file_name);

            if file_path.exists() {
                let data = fs::read(&file_path)
                    .with_context(|| format!("Failed to read file: {:?}", file_path))?;
                Ok(Response::new(200, "OK".to_string())
                    .header("Content-Type", "application/octet-stream".to_string())
                    .body(data))
            } else {
                Ok(Response::new(404, "Not Found".to_string()))
            }
        }

        ("POST", path) if path.starts_with("/files/") => {
            let directory = args.directory.clone().unwrap_or_else(|| ".".into());
            let file_name = path.strip_prefix("/files/").unwrap_or("");
            let file_path = Path::new(&directory).join(file_name);

            fs::write(&file_path, &request.body)
                .with_context(|| format!("Failed to write to file: {:?}", file_path))?;

            Ok(Response::new(201, "Created".to_string()))
        }

        _ => Ok(Response::new(404, "Not Found".to_string())),
    }
}
