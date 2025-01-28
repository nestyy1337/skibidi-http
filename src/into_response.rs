use crate::server::handle_connection::StatusCode;
use std::io::Write;
use std::{collections::HashMap, io::Read};

use thiserror::Error;

use crate::client::client::Request;

pub struct Response {
    status_code: StatusCode,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct ResponseBuilder {
    status_code: Option<StatusCode>,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            status_code: None,
            headers: Some(HashMap::new()),
            body: None,
        }
    }

    pub fn status_code(mut self, code: StatusCode) -> Self {
        self.status_code = Some(code);
        self
    }

    pub fn header(mut self, header: (&str, &str)) -> Self {
        //header is always some
        self.headers
            .as_mut()
            .unwrap()
            .insert(header.0.to_string(), header.1.to_string());
        self
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        self.body = Some(body.to_vec());
        self
    }

    pub fn build(self) -> Response {
        Response {
            status_code: self.status_code.expect("status_code is never none"),
            headers: self.headers.expect("headers is at least empty map"),
            body: self.body,
        }
    }
}

impl Response {
    fn new_with_body(body: String) -> Self {
        Self {
            // statuscode needs default and here we set the default to OK
            status_code: StatusCode::ALL_OK,
            body: Some(body.as_bytes().to_vec()),
            headers: HashMap::new(),
        }
    }
    fn new_with_file(body: Vec<u8>) -> Self {
        Self {
            status_code: StatusCode::ALL_OK,
            body: Some(body),
            headers: HashMap::new(),
        }
    }
    fn add_core_header(&mut self, k: String, v: String) {
        self.headers.insert(k, v);
    }

    fn new() -> Self {
        Self {
            // statuscode needs default and here we set the default to OK
            status_code: StatusCode::ALL_OK,
            body: None,
            headers: HashMap::new(),
        }
    }

    pub fn error() -> Self {
        Self {
            // statuscode needs default and here we set the default to OK
            status_code: StatusCode::NOT_FOUND,
            body: None,
            headers: HashMap::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response: Vec<u8> = Vec::new();

        response.extend_from_slice(self.status_code.as_str().as_bytes());
        response.extend_from_slice(b"\r\n");

        response.extend_from_slice(b"Connection: close\r\n");
        for (k, v) in self.headers.iter() {
            write_header(k, v, &mut response);
        }

        if let Some(body) = &self.body {
            let body_len = body.len();

            write_header(
                "Content-Length",
                body_len.to_string().as_str(),
                &mut response,
            );
            response.extend_from_slice(b"\r\n");
            //write body
            response.extend_from_slice(&body);
        } else {
            response.extend_from_slice(b"\r\n");
        }

        response
    }
}

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        let mut resp = Response::new_with_file(self.to_string().into());
        resp.add_core_header("Content-Type".to_string(), "text/plain".to_string());
        resp
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> Response {
        let mut resp = Response::new_with_file(self.to_string().into());
        resp.add_core_header("Content-Type".to_string(), "text/plain".to_string());
        resp
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let (status, body) = self;
        let mut response = body.into_response();
        response.status_code = status.clone();
        response
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        match self {
            StatusCode::NOT_FOUND => "404 Not Found".into_response(),
            StatusCode::CREATED => "201 Created".into_response(),
            _ => "500 Internal Server Error".into_response(),
        }
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        let mut resp = Response::new_with_file(self);
        resp.add_core_header(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        );
        resp
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let response = Response::error();
        response
    }
}

impl IntoResponse for Result<Response, HandlerError> {
    fn into_response(self) -> Response {
        match self {
            Ok(fine) => fine.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("failed to serve on socket")]
    MainHandlerError,
}

pub trait Handler: Send + Sync + 'static {
    fn call(
        &self,
        req: &Request,
        params: HashMap<String, String>,
    ) -> Result<Response, HandlerError>;
}

impl<F, R> Handler for F
where
    F: Fn(&Request, HashMap<String, String>) -> R + Send + Sync + 'static,
    R: IntoResponse,
{
    fn call(
        &self,
        request: &Request,
        params: HashMap<String, String>,
    ) -> Result<Response, HandlerError> {
        Ok((self)(request, params).into_response())
    }
}

pub trait Handler0: Send + Sync + 'static {
    fn call(&self) -> Result<Response, HandlerError>;
}

impl<F, R> Handler0 for F
where
    F: Fn() -> R + Send + Sync + 'static,
    R: IntoResponse,
{
    fn call(&self) -> Result<Response, HandlerError> {
        Ok((self)().into_response())
    }
}

pub trait HandlerParams: Send + Sync + 'static {
    fn call(&self, params: HashMap<String, String>) -> Result<Response, HandlerError>;
}

impl<F, R> HandlerParams for F
where
    F: Fn(HashMap<String, String>) -> R + Send + Sync + 'static,
    R: IntoResponse,
{
    fn call(&self, params: HashMap<String, String>) -> Result<Response, HandlerError> {
        Ok((self)(params).into_response())
    }
}

pub trait HandlerRequest: Send + Sync + 'static {
    fn call(&self, request: &Request) -> Result<Response, HandlerError>;
}

impl<F, R> HandlerRequest for F
where
    F: Fn(&Request) -> R + Send + Sync + 'static,
    R: IntoResponse,
{
    fn call(&self, request: &Request) -> Result<Response, HandlerError> {
        Ok((self)(request).into_response())
    }
}

fn write_header<'a>(key: &'a str, value: &'a str, response: &'a mut Vec<u8>) {
    write!(response, "{}: {}\r\n", key, value).unwrap();
}
