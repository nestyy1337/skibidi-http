pub mod client;
use client::client::Method;
use client::client::Request;
use into_response::Handler0;
use into_response::HandlerParams;
use into_response::HandlerRequest;
use server::router::MethodTypes;
use std::io::{self, Write};
use tokio::io::AsyncWriteExt;
pub mod into_response;
pub mod server;
use into_response::Handler;
pub use into_response::IntoResponse;
use std::collections::HashMap;
use std::net::TcpListener;

pub enum HandlerTypes {
    ZeroParams((Box<dyn Handler0 + Send + Sync + 'static>, Method)),
    Full((Box<dyn Handler + Send + Sync + 'static>, Method)),
    Body((Box<dyn HandlerRequest + Send + Sync + 'static>, Method)),
    Params((Box<dyn HandlerParams + Send + Sync + 'static>, Method)),
}

impl HandlerTypes {
    fn get_method(&self) -> &Method {
        match self {
            HandlerTypes::ZeroParams((_, method)) => method,
            HandlerTypes::Full((_, method)) => method,
            HandlerTypes::Body((_, method)) => method,
            HandlerTypes::Params((_, method)) => method,
        }
    }
}

impl HandlerTypes {
    pub fn full<F, R>(handler: F, method: Method) -> Self
    where
        F: Fn(Request) -> R + Send + Sync + 'static,
        R: IntoResponse,
    {
        HandlerTypes::Full((Box::new(handler), method))
    }

    pub fn empty<F, R>(handler: F, method: Method) -> Self
    where
        F: Fn() -> R + Send + Sync + 'static,
        R: IntoResponse,
    {
        HandlerTypes::ZeroParams((Box::new(handler), method))
    }

    pub fn body<F, R>(handler: F, method: Method) -> Self
    where
        F: Fn(Request) -> R + Send + Sync + 'static,
        R: IntoResponse,
    {
        HandlerTypes::Body((Box::new(handler), method))
    }

    pub fn params<F, R>(handler: F, method: Method) -> Self
    where
        F: Fn(HashMap<String, String>) -> R + Send + Sync + 'static,
        R: IntoResponse,
    {
        HandlerTypes::Params((Box::new(handler), method))
    }
}

pub trait Methodable: Send + Sync + 'static {
    fn wrap(&self, handler: HandlerTypes) -> Box<HandlerTypes>;
}

pub fn get(
    func: impl IntoResponse + Sync + Send + 'static,
) -> Box<dyn IntoResponse + Sync + Send + 'static> {
    Box::new(func)
}

pub enum TcpListeners {
    Blocking(TcpListener),
    Asynchronous(TcpListener),
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
