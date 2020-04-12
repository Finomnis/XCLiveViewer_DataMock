use log::*;

use super::connection::XCMockConnection;

use tokio;


pub struct XCMockServer {}

impl XCMockServer {
    pub fn new() -> XCMockServer {
        XCMockServer {}
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            match server.accept().await
            {
                Err(e) => error!("ERROR: {}", e),
                Ok((socket, addr)) => {
                    info!("Connected: {}", addr);
                    tokio::spawn(async move {
                        match XCMockConnection::handle(socket).await {
                            Ok(()) => info!("Disconnected: {}", addr),
                            Err(e) => warn!("Connection error: {}: {}", addr, e),
                        }
                    });
                }
            }
        }
    }
}
