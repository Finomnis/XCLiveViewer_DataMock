use log::*;

use tokio;
use tungstenite::Message;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use futures::stream::{SplitStream, SplitSink};
use futures::channel::mpsc;

use crate::utils::{AsyncResult, Json};
use crate::xcmock::XCMockDatabaseInterface;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;

pub struct XCMockConnection {
    send_queue: mpsc::Sender<Message>,
    database_interface: XCMockDatabaseInterface,
}

impl XCMockConnection {
    /// Creates a new websocket connection handler.
    ///
    /// Blocks until the connection has ended or an error has occured.
    pub async fn handle(
        socket: tokio::net::TcpStream,
        database_interface: XCMockDatabaseInterface,
    ) -> AsyncResult<()> {
        let (send_queue, send_queue_sink) = mpsc::channel(1);

        let connection = XCMockConnection {
            send_queue,
            database_interface,
        };

        let (
            websocket_out,
            websocket_in,
        ) = accept_async(socket).await?.split();

        tokio::select! {
        err = connection.send_loop(websocket_out, send_queue_sink) => err,
        err = connection.receive_loop(websocket_in) => err,
        //err = connection.flight_info_sender() => err,
        }
    }

    /// Enqueues the sending of a message
    async fn send(&self, tag: &str, contents: Json) -> AsyncResult<()> {
        self.send_queue.clone().send(Message::Text(String::from(tag))).await?;
        Ok(())
    }

    /// Processes an incoming message.
    async fn process_msg(&self, msg: String) -> AsyncResult<()> {
        self.send("Echo", Json::String(msg)).await?;
        self.send_flight_infos().await?;
        Ok(())
    }

    /// Receive loop. Forwards received messages to self.process_msg.
    async fn receive_loop(&self, mut stream: SplitStream<WebSocket>) -> AsyncResult<()> {
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
    async fn send_loop(
        &self,
        mut stream: SplitSink<WebSocket, Message>,
        mut message_queue: mpsc::Receiver<Message>,
    ) -> AsyncResult<()> {
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

    async fn send_flight_infos(&self) -> AsyncResult<()> {
        // current problem: flight_info_sender sends always, does not respect enabled/disabled decision
        let mut database_interface = self.database_interface.clone();
        let flight_infos = database_interface.get_flight_infos().await?;
        self.send("LiveFlightInfos", flight_infos).await?;
        Ok(())
    }

    async fn flight_info_sender(&self) -> AsyncResult<()> {
        /*let mut flight_info_watch = self.flight_info_watch.clone();
        loop {
            match flight_info_watch.recv().await {
                None => Err("Flight info watch closed.")?,
                Some(msg) => {
                    debug!("Sending new flight infos ...");
                    self.send(msg).await?;
                }
            }
        }*/
        Ok(())
    }
}
