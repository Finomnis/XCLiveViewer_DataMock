use log::*;

use tungstenite::Message;
use tokio_tungstenite::accept_async;
use futures::StreamExt;

pub struct XCMockConnection {
    socket: tokio::net::TcpStream
}

impl XCMockConnection {
    pub fn new(socket: tokio::net::TcpStream) -> XCMockConnection {
        XCMockConnection { socket }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut websocket = accept_async(self.socket).await?;

        loop {
            match websocket.next().await {
                None => Err("Connection lost.")?,
                Some(msg) => {
                    match msg? {
                        Message::Text(text) => info!("Text: {}", text),
                        Message::Binary(data) => info!("Binary: {:?}", data),
                        Message::Ping(_) | Message::Pong(_) => (),
                        Message::Close(_) => break,
                    };
                }
            }
        }

        Ok(())
    }
}
