use log::*;

use tokio;
use tokio::time;
use tungstenite::Message;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use futures::stream::{SplitStream, SplitSink};
use futures::channel::mpsc;
use serde_json::json;
use std::time::Duration;

use crate::utils::{AsyncResult, Json};
use crate::xcmock::XCMockDatabaseInterface;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;

pub struct XCMockConnection {
    send_queue: mpsc::Sender<Message>,
    database_interface: XCMockDatabaseInterface,
    set_flight_infos_enabled: mpsc::Sender<bool>,
}

impl XCMockConnection {
    /// Creates a new websocket connection handler.
    ///
    /// Blocks until the connection has ended or an error has occured.
    pub async fn handle(
        socket: tokio::net::TcpStream,
        database_interface: XCMockDatabaseInterface,
    ) -> AsyncResult<()> {
        let (send_queue, send_queue_stream) = mpsc::channel(0);
        let (set_flight_infos_enabled, flight_infos_enabled) = mpsc::channel(0);

        let connection = XCMockConnection {
            send_queue,
            database_interface,
            set_flight_infos_enabled,
        };

        let (
            websocket_out,
            websocket_in,
        ) = accept_async(socket).await?.split();

        tokio::select! {
            err = connection.send_loop(websocket_out, send_queue_stream) => err,
            err = connection.receive_loop(websocket_in) => err,
            err = connection.flight_info_sender(flight_infos_enabled) => err,
        }
    }

    /// Enqueues the sending of a message
    async fn send(&self, tag: &str, contents: Json) -> AsyncResult<()> {
        let message = json!({
            "tag": tag,
            "contents": contents
        });
        self.send_queue.clone().send(Message::Text(message.to_string())).await?;
        Ok(())
    }

    /// Processes an incoming message.
    async fn process_msg(&self, msg: String) -> AsyncResult<()> {
        self.send("Echo", Json::String(msg)).await?;
        self.set_flight_infos_enabled.clone().send(true).await?;
        Ok(())
    }

    /// Receive loop. Forwards received messages to self.process_msg.
    async fn receive_loop(&self, mut stream: SplitStream<WebSocket>) -> AsyncResult<()> {
        loop {
            let msg = stream.next().await.ok_or("Connection lost.")?;
            match msg? {
                Message::Text(text) => self.process_msg(text).await?,
                Message::Binary(data) => warn!("Received binary message: {:?}", data),
                Message::Ping(_) | Message::Pong(_) => (),
                Message::Close(_) => break,
            };
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
            let msg = message_queue.next().await.ok_or("Send queue closed.")?;
            stream.send(msg).await?;
        }
    }

    async fn flight_info_sender(&self, mut flight_infos_enabled: mpsc::Receiver<bool>) -> AsyncResult<()> {
        let mut database_interface = self.database_interface.clone();
        let mut enabled = false;
        loop {
            if enabled {
                // If already enabled, wait for timer and listen to the enabled setting simultaneously
                let enabled_changed = tokio::select! {
                    // If setting changed, return new setting state and send an update.
                    ret = flight_infos_enabled.next() => {
                        Some(ret.ok_or("Lost connection to flight_infos_enabled setting.")?)
                    }
                    // If setting doesn't change, wait until it is time to send another update
                    _ = time::delay_for(Duration::from_secs(55)) => None
                };

                if let Some(new_enabled_state) = enabled_changed {
                    enabled = new_enabled_state;
                }
            } else {
                // In not enabled, only listen to the enabled setting
                enabled = flight_infos_enabled
                    .next()
                    .await
                    .ok_or("Lost connection to flight_infos_enabled setting.")?;
            }

            if enabled {
                let flight_infos = database_interface.get_flight_infos().await?;
                self.send("LiveFlightInfos", flight_infos).await?;
            }
        }
    }
}
