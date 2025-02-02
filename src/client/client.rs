use std::{collections::HashMap, io::Read, net::TcpStream, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    POST,
    DELETE,
}

#[derive(Error, Debug)]
pub enum MethodParseError {
    #[error("Unknown HTTP method: {0}")]
    Unknown(String),
}

impl FromStr for Method {
    type Err = MethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "DELETE" => Ok(Method::DELETE),
            other => Err(MethodParseError::Unknown(other.to_string())),
        }
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::GET => "GET".to_string(),
            Method::POST => "POST".to_string(),
            Method::DELETE => "DELETE".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    path: String,
    version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(
        method: Method,
        path: &str,
        version: &str,
        headers: HashMap<String, String>,
    ) -> Self {
        let req = Request {
            method,
            path: path.to_string(),
            version: version.to_string(),
            headers: headers,
            body: None,
        };

        req
    }

    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(body)
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        match self.headers.get(key) {
            Some(val) => Some(val.trim()),
            None => None,
        }
    }

    pub fn get_method(&self) -> &Method {
        &self.method
    }
}
pub trait IntoRequest {
    fn into_request(self) -> Request;
}

impl IntoRequest for String {
    fn into_request(self) -> Request {
        Request {
            method: Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

// impl IntoRequest for TcpStream {
//     fn into_request(self) -> Request {
//         let buf: Vec<u8> = Vec::new();
//     }
// }
