use log::*;

use crate::utils::{AsyncResult, Json};

use futures::channel::{mpsc, oneshot};
use futures::{StreamExt, SinkExt};
use std::collections::HashMap;
use crate::data::flight_id::{generate_flight_id, FlightId};
use crate::data::flight_data::FlightData;
use serde_json::to_value;

type XCMockDatabaseCommand = (oneshot::Sender<Json>, XCMockDatabaseOperation);

pub struct XCMockDatabase {
    command_queue_tx: mpsc::Sender<XCMockDatabaseCommand>,
    command_queue: mpsc::Receiver<XCMockDatabaseCommand>,
    flights: HashMap<FlightId, FlightData>,
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

        // Todo remove
        let mut flights = HashMap::new();
        flights.insert(generate_flight_id(), FlightData::random());

        XCMockDatabase {
            command_queue_tx,
            command_queue,
            flights,
        }
    }

    pub fn create_interface(&self) -> XCMockDatabaseInterface {
        XCMockDatabaseInterface {
            database: self.command_queue_tx.clone()
        }
    }

    pub async fn run(mut self) -> AsyncResult<()> {
        loop {
            let (callback, operation) = self.command_queue.next().await.ok_or("All connections to the database closed!")?;
            let op_result = self.execute_operation(operation).await?;
            callback.send(op_result).or(Err("Unable to submit operation result."))?
        }
    }

    fn get_flight_infos(&mut self) -> AsyncResult<Json> {
        let mut flight_infos = vec![];
        for (flight_id, flight_data) in &self.flights {
            flight_infos.push(Json::Array(vec![
                Json::String(flight_id.clone()),
                to_value(flight_data.to_json())?
            ]));
        }
        Ok(Json::Array(flight_infos))
    }

    async fn execute_operation(&mut self, operation: XCMockDatabaseOperation) -> AsyncResult<Json> {
        debug!("Executing database command: {:?}", operation);
        let result = match operation {
            XCMockDatabaseOperation::GetFlightInfos => self.get_flight_infos()?
        };
        Ok(result)
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
