use skibidi_http::server::handle_connection::StatusCode;
use skibidi_http::server::router::Router;
use skibidi_http::server::server::Server;
use std::collections::HashMap;
use std::fs;
use tokio::net::TcpListener;

use skibidi_http::client::client::{Method, Request};
use skibidi_http::into_response::{HandlerError, Response, ResponseBuilder};
use skibidi_http::{HandlerTypes, IntoResponse};

// shit without macros is pain
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    // building the router
    let router = Router::builder()
        //why do i have to suffer through lack of specialization in stable rust
        .route("/", HandlerTypes::full(simple_handler, Method::GET))
        // since im not allowed to use macros we need to specify the handler type
        // this would allow the handler fn to access parts of the request
        // also we have to specify allowed method on a route
        .route("/user-agent", HandlerTypes::full(user_agent, Method::GET))
        .route("/user-agent", HandlerTypes::full(user_agent, Method::POST))
        .route(
            "/echo/{str}",
            HandlerTypes::params(respond_with_body_handler, Method::POST),
        )
        .route("/empty", HandlerTypes::empty(test_hander, Method::POST))
        .route(
            "/files/{filename}",
            HandlerTypes::params(respond_with_file, Method::POST),
        )
        .route("/ill", HandlerTypes::empty(complicated, Method::GET))
        .build();

    let service = router.into_service();
    let mut server = Server::new(listener, service);
    server.serve().await.unwrap();
}

fn simple_handler(request: Request) -> Result<Response, HandlerError> {
    if let Some(param) = request.headers.values().next() {
        return Ok(param.clone().into_response());
    }
    Ok(().into_response())
}

// acessing headers of the request
fn user_agent(request: Request) -> Result<Response, HandlerError> {
    let step = request.get_header("User-Agent");
    if let Some(header) = step {
        return Ok(header.into_response());
    }

    Ok(().into_response())
}

// responding with a type implementing IntoResponse trait
fn test_hander() -> impl IntoResponse {
    HandlerError::MainHandlerError
}

// acessing path parameter
fn respond_with_body_handler(map: HashMap<String, String>) -> Result<Response, HandlerError> {
    let body = map.get("str");
    match body {
        Some(bod) => Ok(bod.clone().into_response()),
        None => Err(HandlerError::MainHandlerError),
    }
}

// converting to concrete Response to allow different types under the hood
fn respond_with_file(map: HashMap<String, String>) -> Response {
    let file_name = map.get("filename").unwrap();
    let path = "./";

    let file_path = format!("{}{}", path, file_name);
    match fs::read(&file_path) {
        Ok(contents) => (StatusCode::ALL_OK, contents).into_response(), // (StatusCode, Vec<u8>)
        Err(_) => (StatusCode::NOT_FOUND, "pozdro nie ma tu wstepu").into_response(), // (StatusCode, &str)
    }
}

// using builder
fn complicated() -> Response {
    ResponseBuilder::new()
        .header(("ayaya", "illness"))
        .status_code(StatusCode::FORBIDDEN)
        .build()
}
