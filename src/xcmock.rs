use tokio;
use tokio_tungstenite::accept_async;

use log::*;

pub struct XCMock {}

impl XCMock {
    pub fn new() -> XCMock {
        XCMock {}
    }

    async fn process_socket(&self, socket: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>>
    {
        let _websocket = accept_async(socket).await?;

        Ok(())
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            let (socket, addr) = server.accept().await?;
            info!("Connected: {}", addr);
            match self.process_socket(socket).await {
                Ok(()) => info!("Disconnected: {}", addr),
                Err(e) => warn!("Connection error: {}: {}", addr, e),
            }
        }
    }
}
