mod xcmock;

use xcmock::XCMock;

use async_ctrlc::CtrlC;
use std::process::exit;
use log::*;

async fn wait_for_ctrl_c() -> Result<(), Box<dyn std::error::Error>> {
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

    let xc_mock = XCMock::new();

    // Combine server, data source and ctrl-c handler in one select.
    // Ctrl-C is the only future that can actually trigger.
    // Therefore, Ctrl-C cancels the other two futures, shutting down the
    // program gracefully.
    let result = tokio::select! {
        err = wait_for_ctrl_c() => err,
        err = xc_mock.run() => err
    };

    match result {
        Ok(()) => info!("Server stopped."),
        Err(e) => {
            error!("Server error: {}", e);
            exit(1)
        }
    }
}
