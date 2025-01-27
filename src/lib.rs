pub mod client;
use std::io::{self, Write};
use tokio::io::AsyncWriteExt;
pub mod into_response;
pub mod server;
use crate::handle_connection::{handle_connection, handle_connection_blocking};
pub use into_response::IntoResponse;
use into_response::{Handler, Response};
use server::handle_connection;
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use thiserror::Error;
use tokio::runtime::Builder;

pub struct RouterBuilder {
    routes: Vec<(&'static str, Box<dyn Handler + Send + Sync>)>,
}

pub struct Router {
    inner: Vec<(&'static str, Box<dyn Handler + Send + Sync>)>,
}

#[derive(Clone)]
pub struct RouterService {
    pub router: Arc<Router>,
}

pub struct RouteMatch<'a> {
    pub handler: &'a Box<dyn Handler + Send + Sync>,
    pub params: HashMap<String, String>,
}

pub struct Server {
    listener: tokio::net::TcpListener,
    router: RouterService,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("failed to serve on socket")]
    ServerErr,
}
#[derive(Error, Debug)]
pub enum RouterError {
    #[error("failed to find appropriate route")]
    PathNotFound,
}

type Route = (&'static str, Box<dyn Handler + Send + Sync>);
type HandlerType = Box<dyn Handler + Send + Sync>;

impl RouterBuilder {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route(mut self, path: &'static str, handler: HandlerType) -> Self {
        self.routes.push((path, handler));
        self
    }

    pub fn build(self) -> Router {
        Router { inner: self.routes }
    }
}

impl Router {
    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }

    pub fn into_service(self) -> RouterService {
        RouterService {
            router: Arc::new(self),
        }
    }
}

impl Router {
    fn matches(&self, path: &str) -> Option<RouteMatch> {
        // let normalized = normalize_path(path);

        self.inner.iter().find_map(|(pattern, handler)| {
            let path_pattern = PatternPath::from_path(pattern);
            if path_pattern.matches(&path) {
                Some(RouteMatch {
                    handler,
                    params: path_pattern.extract_params(&path),
                })
            } else {
                None
            }
        })
    }
}

fn normalize_path(path: &str) -> String {
    let segments = path
        .split("")
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();
    format!("/{}", segments.join("/"))
}

pub enum TcpListeners {
    Blocking(TcpListener),
    Asynchronous(TcpListener),
}

impl Server {
    pub fn new(listener: tokio::net::TcpListener, router: RouterService) -> Self {
        Self { listener, router }
    }

    // pub fn serve_blocking(&self) -> Result<(), ServerError> {
    //     for stream in self.listener.incoming() {
    //         match stream {
    //             Ok(stream) => handle_connection_blocking(stream, self.router.clone()),
    //             Err(_) => return Err(ServerError::ServerErr),
    //         }
    //     }
    //     Ok(())
    // }

    pub async fn serve_async(&self, listener: &tokio::net::TcpListener) -> Result<(), ServerError> {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let router = self.router.clone();
                    tokio::spawn(handle_connection(stream, router));
                }
                Err(_) => return Err(ServerError::ServerErr),
            };
        }
    }
    // pub fn serve(&mut self) -> Result<(), ServerError> {
    //     match &self.listener {
    //         TcpListeners::Blocking(listener) => self.serve_blocking(listener),
    //         TcpListeners::Asynchronous(listener) => self.serve_async(listener),
    //     }
    // }
    pub async fn serve(&mut self) -> Result<(), ServerError> {
        self.serve_async(&self.listener).await
    }
}

#[derive(Debug)]
pub struct PatternPath {
    segments: Vec<PathSegment>,
}

#[derive(Debug)]
enum PathSegment {
    Static(String),
    Parameter(String),
}

impl PatternPath {
    fn from_path(path: &str) -> Self {
        let segments = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if segment.starts_with("{") && segment.ends_with("}") {
                    PathSegment::Parameter(segment[1..segment.len() - 1].to_string())
                } else {
                    //normal path
                    PathSegment::Static(segment.to_string())
                }
            })
            .collect();
        PatternPath { segments }
    }

    fn matches(&self, path: &str) -> bool {
        let path_segments: Vec<_> = path.split("/").filter(|s| !s.is_empty()).collect();

        if path_segments.len() != self.segments.len() {
            return false;
        }

        self.segments
            .iter()
            .zip(path_segments)
            .all(|(pattern, segment)| match pattern {
                PathSegment::Static(s) => s == segment,
                PathSegment::Parameter(_) => true,
            })
    }

    pub fn extract_params(&self, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let path_segments: Vec<_> = path.split("/").filter(|s| !s.is_empty()).collect();

        for (pattern, path_seg) in self.segments.iter().zip(path_segments) {
            if let PathSegment::Parameter(name) = pattern {
                params.insert(name.to_string(), path_seg.to_string());
            }
        }
        params
    }
}

trait ResponseWriter {
    fn write_response<T: AsRef<[u8]>>(&mut self, response: T) -> io::Result<()>;
}

impl ResponseWriter for std::net::TcpStream {
    fn write_response<T: AsRef<[u8]>>(&mut self, response: T) -> io::Result<()> {
        self.write_all(response.as_ref())?;
        self.flush()?;
        Ok(())
    }
}

#[async_trait::async_trait]
trait AsyncResponseWriter {
    async fn write_response<T: AsRef<[u8]> + Send>(&mut self, response: T) -> io::Result<()>;
}

#[async_trait::async_trait]
impl AsyncResponseWriter for tokio::net::TcpStream {
    async fn write_response<T: AsRef<[u8]> + Send>(&mut self, response: T) -> io::Result<()> {
        self.write_all(response.as_ref()).await?;
        self.flush().await?;
        Ok(())
    }
}

fn write_blocking<T>(stream: &mut std::net::TcpStream, response: T) -> io::Result<()>
where
    T: AsRef<[u8]>,
{
    ResponseWriter::write_response(stream, response)
}

async fn write_async<T>(stream: &mut tokio::net::TcpStream, response: T) -> io::Result<()>
where
    T: AsRef<[u8]> + Send,
{
    println!("response: {:?}", String::from_utf8_lossy(response.as_ref()));
    AsyncResponseWriter::write_response(stream, response).await
}
