use crate::client::parse::parse_http;
use crate::client::parse::parse_http_blocking;
use crate::into_response::Response;
use crate::server::router::RouterService;
use crate::write_async;
use crate::write_blocking;
use crate::HandlerTypes;
use crate::IntoResponse;
use std::net::TcpStream;

pub fn handle_connection_blocking(mut stream: TcpStream, service: RouterService) {
    loop {
        match parse_http_blocking(&mut stream) {
            Ok(request) => {
                match service.router.matches(request.get_path()) {
                    Some(route_match) => {
                        if request.get_method() == route_match.methods.to_string() {
                            let method_not_allowed_response =
                                StatusCode::METHOD_NOT_ALLOWED.into_response().to_bytes();

                            if let Err(e) =
                                write_blocking(&mut stream, &method_not_allowed_response)
                            {
                                eprintln!("ERRORED WITH: {:?}", e);
                                break;
                            } else {
                                break;
                            }
                        }

                        let resp = match &route_match.handler {
                            HandlerTypes::ZeroParams(a) => a.0.call().unwrap().to_bytes(),
                            HandlerTypes::Params(a) => {
                                a.0.call(request.headers).unwrap().to_bytes()
                            }
                            HandlerTypes::Body(a) => a.0.call(request).unwrap().to_bytes(),
                            HandlerTypes::Full(a) => a.0.call(request).unwrap().to_bytes(),
                        };

                        // .call(&request, route_match.params)
                        // .unwrap()
                        // .to_bytes();

                        if let Err(e) = write_blocking(&mut stream, &resp) {
                            eprintln!("ERRORED WITH: {:?}", e);
                            break;
                        } else {
                            break;
                        }
                    }
                    None => {
                        // Handle 404
                        if let Err(_) =
                            write_blocking(&mut stream, StatusCode::INTERNAL_SERVER_ERROR.as_str())
                        {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("ERRORED INSIDE PARSING: {:?}", e);
                let _ = write_blocking(&mut stream, StatusCode::INTERNAL_SERVER_ERROR.as_str());
                break;
            }
        }
    }
}

pub async fn handle_connection(mut stream: tokio::net::TcpStream, service: RouterService) {
    loop {
        match parse_http(&mut stream).await {
            Ok(request) => {
                match service.router.matches(request.get_path()) {
                    Some(route_match) => {
                        println!(
                            "{} : {}",
                            request.get_method(),
                            route_match.methods.to_string()
                        );
                        println!("path: {:?}", request.get_path());

                        if request.get_method() == route_match.methods.to_string() {
                            println!("shouldnt go there");
                            let resp = match &route_match.handler {
                                HandlerTypes::ZeroParams(a) => a.0.call().unwrap().to_bytes(),
                                HandlerTypes::Params(a) => {
                                    a.0.call(request.headers).unwrap().to_bytes()
                                }
                                HandlerTypes::Body(a) => a.0.call(request).unwrap().to_bytes(),
                                HandlerTypes::Full(a) => a.0.call(request).unwrap().to_bytes(),
                            };

                            if let Err(e) = write_async(&mut stream, &resp).await {
                                eprintln!("ERRORED WITH: {:?}", e);
                                break;
                            } else {
                                break;
                            }
                        } else {
                            println!("should be HERE");
                            let method_not_allowed_response =
                                StatusCode::METHOD_NOT_ALLOWED.into_response().to_bytes();

                            if let Err(e) =
                                write_async(&mut stream, &method_not_allowed_response).await
                            {
                                println!("ERRORED WITH: {:?}", e);
                                break;
                            } else {
                                break;
                            }
                        }
                    }
                    None => {
                        //Handle 404
                        println!("route not matched");
                        let _ = write_async(&mut stream, StatusCode::NOT_FOUND.as_str()).await;
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("ERRORED INSIDE PARSING: {:?}", e);
                let _ = write_async(&mut stream, StatusCode::INTERNAL_SERVER_ERROR.as_str()).await;
                break;
            }
        }
    }
}

#[derive(Clone)]
pub enum StatusCode {
    ALL_OK,
    INTERNAL_SERVER_ERROR,
    NOT_FOUND,
    CREATED,
    ACCEPTED,
    BAD_REQUEST,
    UNAUTHORIZED,
    FORBIDDEN,
    METHOD_NOT_ALLOWED,
}

impl StatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusCode::ALL_OK => "HTTP/1.1 200 OK\r\n\r\n",
            StatusCode::INTERNAL_SERVER_ERROR => "HTTP/1.1 500 Internal Server Error\r\n\r\n",
            StatusCode::NOT_FOUND => "HTTP/1.1 404 Not Found\r\n\r\n",
            StatusCode::CREATED => "HTTP/1.1 201 Created\r\n\r\n",
            StatusCode::ACCEPTED => "HTTP/1.1 202 Accepted\r\n\r\n",
            StatusCode::BAD_REQUEST => "HTTP/1.1 400 Bad Request\r\n\r\n",
            StatusCode::UNAUTHORIZED => "HTTP/1.1 401 Unauthorized\r\n\r\n",
            StatusCode::FORBIDDEN => "HTTP/1.1 403 Forbidden\r\n\r\n",
            StatusCode::METHOD_NOT_ALLOWED => "HTTP/1.1 405 Method Not Allowed\r\n\r\n",
        }
    }
}
