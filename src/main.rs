use clap::Parser;
use log::{
    error,
    info,
};
use tokio::time::timeout;

use crate::p2p::messages::{
    Service,
    Services,
};

mod cli;
mod p2p;

#[tokio::main]
async fn main() {
    env_logger::init();

    const BITCOIN_PROTOCOL_VERSION: i32 = 70015;

    const APP_NAME: &str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

    let config = p2p::NodeConfig {
        version: BITCOIN_PROTOCOL_VERSION,
        services: Services::new(&[Service::Network]),
        user_agent: format!("{}/{}/", APP_NAME, APP_VERSION),
        start_height: 1,
        relay: false,
    };

    let node = p2p::Node::new(config);
    let args = cli::Arguments::parse();

    for address in args.addresses {
        info!("Performing a handshake with {}", address);

        match timeout(args.timeout, node.handshake(args.network, address)).await {
            Ok(v) => match v {
                Ok(node_config) => info!(
                    "Handshake successfully performed, node at {}: {}",
                    address, node_config
                ),
                Err(e) => error!("Error occurred during handshake: {}", e),
            },
            Err(e) => {
                error!("Timeout of {} ms exceeded: {}", args.timeout.as_millis(), e)
            }
        }
    }
}
