use log::*;

use super::connection::XCMockConnection;
use super::XCMockDatabaseInterface;
use crate::utils::AsyncResult;

use tokio;

pub struct XCMockServer {
    database_interface: XCMockDatabaseInterface
}

impl XCMockServer {
    pub fn new(database_interface: XCMockDatabaseInterface) -> XCMockServer {
        XCMockServer {
            database_interface
        }
    }

    pub async fn run(self) -> AsyncResult<()> {
        let mut server = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        loop {
            match server.accept().await
            {
                Err(e) => error!("ERROR: {}", e),
                Ok((socket, addr)) => {
                    info!("Connected: {}", addr);
                    let database_interface = self.database_interface.clone();
                    tokio::spawn(async move {
                        match XCMockConnection::handle(socket, database_interface).await {
                            Ok(()) => info!("Disconnected: {}", addr),
                            Err(e) => error!("Connection error: {}: {}", addr, e),
                        }
                    });
                }
            }
        }
    }
}
