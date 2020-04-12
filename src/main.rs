mod xcmock;
mod utils;

use xcmock::{XCMockServer, XCMockDataGen};

use async_ctrlc::CtrlC;
use std::process::exit;
use log::*;

use crate::utils::AsyncResult;

async fn wait_for_ctrl_c() -> AsyncResult {
    let ctrlc = CtrlC::new()?;
    ctrlc.await;
    info!("Stopping server ...");
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let xc_server = XCMockServer::new();
    let xc_data_gen = XCMockDataGen::new();

    // Combine server, data source and ctrl-c handler in one select.
    // Ctrl-C is the only future that can actually trigger.
    // Therefore, Ctrl-C cancels the other two futures, shutting down the
    // program gracefully.
    let result = tokio::select! {
        err = wait_for_ctrl_c() => err,
        err = xc_server.run() => err,
        err = xc_data_gen.run() => err,
    };

    match result {
        Ok(()) => info!("Server stopped."),
        Err(e) => {
            error!("Server error: {}", e);
            exit(1)
        }
    }
}
