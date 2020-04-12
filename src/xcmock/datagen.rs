use log::*;

use tokio::sync::watch;
use tokio::time;

use std::time::Duration;
use crate::utils::AsyncResult;

pub struct XCMockDataGen {
    fight_infos_tx: watch::Sender<String>,
    fight_infos_rx: watch::Receiver<String>,
}

impl XCMockDataGen {
    pub fn new() -> XCMockDataGen {
        let (fight_infos_tx, fight_infos_rx) = watch::channel(String::from("{{\"tag\":\"LiveFlightInfos\",\"contents\":[]}}"));
        XCMockDataGen {
            fight_infos_tx,
            fight_infos_rx,
        }
    }

    pub fn get_flight_infos_watch(self) -> watch::Receiver<String>
    {
        self.fight_infos_rx.clone()
    }

    pub async fn run(self) -> AsyncResult {
        loop {
            time::delay_for(Duration::from_secs(1)).await;
            println!("Data: beep.");
        }
    }
}
