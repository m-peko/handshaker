use std::net::SocketAddrV4;

use log::info;

use crate::p2p::messages::Services;

pub struct NodeConfig {
    /// Protocol version used by the node
    pub version: i32,
    /// Features to be enabled for the connection
    pub services: Services,
    /// User agent
    pub user_agent: String,
    /// Last block received by the emitting node
    pub start_height: i32,
    /// Whether the remote peer should announce relayed transactions or not
    pub relay: bool,
}

pub struct Node {
    /// Configuration set at the application start
    config: NodeConfig,
}

impl Node {
    pub fn new(config: NodeConfig) -> Self {
        Self { config }
    }

    pub async fn handshake(&self, address: SocketAddrV4) -> Result<(), String> {
        info!("Performing a handshake with {}", address);
        Ok(())
    }
}
