use tokio;
use tokio_tungstenite::accept_async;

use log::*;
use futures::StreamExt;

struct XCMockConnection {
    socket: tokio::net::TcpStream
}

impl XCMockConnection {
    fn new(socket: tokio::net::TcpStream) -> XCMockConnection {
        XCMockConnection { socket }
    }

    async fn run(self) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut websocket = accept_async(self.socket).await?;
        
        loop {
            let msg = websocket.next().await;
            match msg {
                None => Err("Connection lost.")?,
                Some(msg) => {
                    let msg = msg?;
                    //match msg {}
                    info!("MSG: {:?}", msg);
                }
            }
        }

        Ok(())
    }
}

pub struct XCMock {}

impl XCMock {
    pub fn new() -> XCMock {
        XCMock {}
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            match server.accept().await
            {
                Err(e) => error!("ERROR: {}", e),
                Ok((socket, addr)) => {
                    info!("Connected: {}", addr);
                    let connection_handler = XCMockConnection::new(socket);
                    tokio::spawn(async move {
                        match connection_handler.run().await {
                            Ok(()) => info!("Disconnected: {}", addr),
                            Err(e) => warn!("Connection error: {}: {}", addr, e),
                        }
                    });
                }
            }
        }
    }
}
