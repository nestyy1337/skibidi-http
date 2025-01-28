use skibidi_http::server::handle_connection::StatusCode;
use std::collections::HashMap;
use std::fs;
use tokio::net::TcpListener;

use skibidi_http::client::client::Request;
use skibidi_http::into_response::{HandlerError, Response};
use skibidi_http::{get, HandlerTypes};
use skibidi_http::{IntoResponse, Router, Server};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    let router = Router::builder()
        //why do i have to suffer through lack of specialization on stable rust
        // .route("/", HandlerTypes::Full(Box::new(simple_handler)))
        // .route("/user-agent", HandlerTypes::Full(Box::new(simple_handler)))
        .route("/echo/{str}", HandlerTypes::full(respond_with_body_handler))
        // .route("/empty", HandlerTypes::Full(Box::new(test_hander)))
        // .route(
        //     "/files/{filename}",
        //     HandlerTypes::Params(Box::new(respond_with_file)),
        // )
        .build();

    let service = router.into_service();
    let mut server = Server::new(listener, service);
    server.serve().await.unwrap();
}

fn simple_handler(
    request: &Request,
    map: HashMap<String, String>,
) -> Result<Response, HandlerError> {
    println!("triggered handler");
    if let Some(param) = map.values().next() {
        return Ok(param.clone().into_response());
    }
    let step = request.get_header("User-Agent");
    if let Some(header) = step {
        return Ok(header.into_response());
    }
    Ok(().into_response())
}

fn test_hander(request: &Request, map: HashMap<String, String>) -> impl IntoResponse {
    HandlerError::MainHandlerError
}

fn respond_with_body_handler(
    request: &Request,
    map: HashMap<String, String>,
) -> Result<Response, HandlerError> {
    let body = map.get("str");
    match body {
        Some(bod) => Ok(bod.clone().into_response()),
        None => Err(HandlerError::MainHandlerError),
    }
}

fn respond_with_file(map: HashMap<String, String>) -> Response {
    let file_name = map.get("filename").unwrap();
    let path = "./";

    let file_path = format!("{}{}", path, file_name);
    match fs::read(&file_path) {
        Ok(contents) => (StatusCode::ALL_OK, contents).into_response(), // (StatusCode, Vec<u8>)
        Err(_) => (StatusCode::NOT_FOUND, "pozdro nie ma tu wstepu").into_response(), // (StatusCode, &str)
    }
}
