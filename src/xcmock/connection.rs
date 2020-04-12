use log::*;

use tungstenite::Message;
use tokio_tungstenite::accept_async;
use futures::StreamExt;

pub struct XCMockConnection {}

impl XCMockConnection {
    pub fn new() -> XCMockConnection {
        XCMockConnection {}
    }

    pub async fn process_msg(&self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("Text: {}", msg);
        Ok(())
    }

    pub async fn run(self, socket: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut websocket = accept_async(socket).await?;

        loop {
            match websocket.next().await {
                None => Err("Connection lost.")?,
                Some(msg) => {
                    match msg? {
                        Message::Text(text) => self.process_msg(text).await?,
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
