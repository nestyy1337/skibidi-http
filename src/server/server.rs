use thiserror::Error;

use super::{handle_connection::handle_connection, router::RouterService};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("failed to serve on socket")]
    ServerErr,
}

pub struct Server {
    listener: tokio::net::TcpListener,
    router: RouterService,
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
