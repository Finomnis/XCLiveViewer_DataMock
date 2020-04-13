use log::*;

use crate::utils::{AsyncResult, Json};

use futures::channel::{mpsc, oneshot};
use futures::{StreamExt, SinkExt};

type XCMockDatabaseCommand = (oneshot::Sender<Json>, XCMockDatabaseOperation);

pub struct XCMockDatabase {
    command_queue_tx: mpsc::Sender<XCMockDatabaseCommand>,
    command_queue: mpsc::Receiver<XCMockDatabaseCommand>,
}

#[derive(Clone)]
pub struct XCMockDatabaseInterface {
    database: mpsc::Sender<XCMockDatabaseCommand>,
}

#[derive(Debug)]
enum XCMockDatabaseOperation {
    GetFlightInfos
}

impl XCMockDatabase {
    pub fn new() -> XCMockDatabase {
        let (command_queue_tx, command_queue) = mpsc::channel(1);
        XCMockDatabase {
            command_queue_tx,
            command_queue,
        }
    }

    pub fn create_interface(&self) -> XCMockDatabaseInterface {
        XCMockDatabaseInterface {
            database: self.command_queue_tx.clone()
        }
    }

    pub async fn run(mut self) -> AsyncResult<()> {
        loop {
            match self.command_queue.next().await {
                None => Err("All connections to the database closed!")?,
                Some((callback, operation)) => {
                    let op_result = self.execute_operation(operation).await?;
                    callback.send(op_result).or(Err("Unable to submit operation result."))?
                }
            };
        }
    }

    async fn execute_operation(&mut self, operation: XCMockDatabaseOperation) -> AsyncResult<Json> {
        debug!("Executing database command: {:?}", operation);
        Ok(Json::Null)
    }
}

impl XCMockDatabaseInterface {
    async fn execute(&mut self, operation: XCMockDatabaseOperation) -> AsyncResult<Json>
    {
        let (result_cb, result) = oneshot::channel();
        self.database.send((result_cb, operation)).await?;
        Ok(result.await?)
    }

    pub async fn get_flight_infos(&mut self) -> AsyncResult<Json> {
        Ok(self.execute(XCMockDatabaseOperation::GetFlightInfos).await?)
    }
}
