use log::*;

use tokio;
use tungstenite::Message;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use futures::stream::{SplitStream, SplitSink};
use futures::channel::mpsc;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;

pub struct XCMockConnection {
    send_queue: mpsc::Sender<Message>
}

impl XCMockConnection {
    /// Creates a new websocket connection handler.
    ///
    /// Blocks until the connection has ended or an error has occured.
    pub async fn handle(socket: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let (send_queue, send_queue_sink) = mpsc::channel(1);

        let connection = XCMockConnection {
            send_queue,
        };

        let (
            websocket_out,
            websocket_in,
        ) = accept_async(socket).await?.split();

        tokio::select! {
            err = connection.send_loop(websocket_out, send_queue_sink) => err,
            err = connection.receive_loop(websocket_in) => err,
        }
    }

    /// Enqueues the sending of a message
    async fn send(&self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        self.send_queue.clone().send(Message::Text(msg)).await?;
        Ok(())
    }

    /// Processes an incoming message.
    async fn process_msg(&self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Received: {}", msg);
        self.send(msg).await?;
        Ok(())
    }

    /// Receive loop. Forwards received messages to self.process_msg.
    async fn receive_loop(&self, mut stream: SplitStream<WebSocket>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match stream.next().await {
                None => Err("Connection lost.")?,
                Some(msg) => {
                    match msg? {
                        Message::Text(text) => self.process_msg(text).await?,
                        Message::Binary(data) => warn!("Received binary message: {:?}", data),
                        Message::Ping(_) | Message::Pong(_) => (),
                        Message::Close(_) => break,
                    };
                }
            }
        }
        Ok(())
    }

    /// Send loop. Forwards messages from message_queue to socket.
    async fn send_loop(&self,
                       mut stream: SplitSink<WebSocket, Message>,
                       mut message_queue: mpsc::Receiver<Message>)
                       -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let next = message_queue.next().await;
            match next {
                None => Err("Send queue closed.")?,
                Some(msg) => {
                    stream.send(msg).await?;
                }
            }
        }
    }
}
