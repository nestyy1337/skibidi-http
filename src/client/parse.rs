use super::client::{Method, Request};
use memchr::memmem::{self, FindIter};
use std::collections::HashMap;
use std::io::{self, BufReader, Read};
use tokio::io::Interest;

use std::net::TcpStream;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse HTTP headers")]
    HearderError,
    #[error("Failed to parse HTTP method")]
    MethodError,
    #[error("Failed to parse Path,")]
    PathError,
    #[error("Failed to parse Agent")]
    AgentError,
    #[error("Closed connection")]
    CloseConn,
}

//one unique connection between two sockets
pub fn parse_http_blocking(stream: &mut TcpStream) -> Result<Request, ParseError> {
    let mut v: Vec<u8> = Vec::with_capacity(1024);
    loop {
        let mut local = [0; 2024];
        let read = stream.read(&mut local).unwrap();
        println!("REQ: {:?}", String::from_utf8_lossy(&local[..read]));
        v.extend_from_slice(&local[..read]);

        if read == 0 {
            eprintln!("terminating socket con");
            return Err(ParseError::CloseConn);
        }
        if read > local.len() {
            eprintln!("packet sent was too big");
            return Err(ParseError::CloseConn);
        }

        let mut finder = memmem::find_iter(&v, b"\r\n\r\n");
        if let Some(pos) = finder.nth(0) {
            let header = parse_header(&v[..pos]);
            match header {
                Ok(head) => return Ok(head),
                Err(_) => return Err(ParseError::HearderError),
            };
        }
    }
}

fn parse_header(buf: &[u8]) -> Result<Request, ParseError> {
    let header = String::from_utf8_lossy(&buf);

    let request_line = header.lines().nth(0).ok_or(ParseError::HearderError)?;
    println!("line: {}", request_line);
    let mut parts = request_line.split_whitespace();

    let method_str = parts.next().ok_or(ParseError::HearderError)?;
    let path = parts.next().ok_or(ParseError::PathError)?.to_string();
    let version = parts.next().ok_or(ParseError::HearderError)?.to_string();
    let method = Method::from_str(&method_str).map_err(|_| ParseError::HearderError)?;

    let mut hmap = HashMap::new();
    for theader in header.lines().skip(1) {
        println!("dded: {}", theader);
        let (k, v) = theader.split_once(":").ok_or(ParseError::HearderError)?;
        hmap.insert(k.to_string(), v.to_string());
        println!("added: {}", theader);
    }

    Ok(Request::new(method, &path, &version, hmap))
}

pub async fn parse_http(stream: &mut tokio::net::TcpStream) -> Result<Request, ParseError> {
    let mut v: Vec<u8> = Vec::with_capacity(1024);
    loop {
        let ready = stream
            .ready(Interest::READABLE)
            .await
            .map_err(|_| ParseError::CloseConn)?;

        if ready.is_readable() {
            let mut local = [0; 2024];
            match stream.try_read(&mut local) {
                Ok(n) => {
                    println!("REQ: {:?}", String::from_utf8_lossy(&local[..n]));
                    v.extend_from_slice(&local[..n]);

                    if n == 0 {
                        eprintln!("terminating socket con");
                        return Err(ParseError::CloseConn);
                    }
                    if n > local.len() {
                        eprintln!("packet sent was too big");
                        return Err(ParseError::CloseConn);
                    }

                    let mut finder = memmem::find_iter(&v, b"\r\n\r\n");
                    if let Some(pos) = finder.nth(0) {
                        let header = parse_header(&v[..pos]);
                        match header {
                            Ok(head) => return Ok(head),
                            Err(_) => return Err(ParseError::HearderError),
                        };
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => return Err(ParseError::CloseConn),
            }
        }
    }
}
