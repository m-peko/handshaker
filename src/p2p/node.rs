use std::{
    fmt::{
        Display,
        Formatter,
    },
    net::{
        SocketAddr,
        SocketAddrV4,
    },
};

use log::{
    error,
    info,
    warn,
};
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
};

use crate::p2p::{
    messages::{
        calculate_checksum,
        compose,
        Codec,
        CodecError,
        Command,
        MessageHeader,
        Network,
        PingMessage,
        PongMessage,
        Services,
        VerackMessage,
        VersionMessage,
    },
    ConnectionError,
};

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

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            version: 0,
            services: Services::empty(),
            user_agent: String::new(),
            start_height: 0,
            relay: false,
        }
    }
}

impl Display for NodeConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "version: {}, services: {}, user agent: {}, start height: {}, relay: {}",
            self.version, self.services, self.user_agent, self.start_height, self.relay
        )
    }
}

pub struct Node {
    /// Configuration set at the application start
    config: NodeConfig,
}

impl Node {
    pub fn new(config: NodeConfig) -> Self {
        Self { config }
    }

    /// Performs a handshake between NodeA and NodeB in the following way:
    /// - NodeA initiating the handshake establishes TCP connection
    /// - NodeA sends Version message and expects Verack version
    /// - NodeB sends back Version message and expects Verack version
    /// - handshake is successfully performed
    ///
    /// - Ping and Pong messages are used to confirm TCP connection is valid
    ///
    /// Returns configuration of the node with which the handshake was performed.
    pub async fn handshake(
        &self,
        network: Network,
        address: SocketAddrV4,
    ) -> Result<NodeConfig, ConnectionError> {
        let mut other_node_config: NodeConfig = Default::default();

        let mut socket = TcpStream::connect(address)
            .await
            .map_err(|_| ConnectionError::ConnectionRefusedError)?;

        let version_data = compose(
            network,
            Command::Version,
            VersionMessage::new(SocketAddr::from(address), &self.config),
        );
        socket
            .write_all(&version_data[..])
            .await
            .map_err(|_| ConnectionError::IOError)?;

        loop {
            let mut buffer = [0; 4096];
            match socket
                .read(&mut buffer)
                .await
                .map_err(|_| ConnectionError::IOError)?
            {
                0 => return Err(ConnectionError::ConnectionHangUp),
                n => {
                    let mut data = &buffer[..n];

                    let header = match MessageHeader::decode(&mut data) {
                        Ok(v) => v,
                        Err(e) => match e {
                            CodecError::InvalidBytesError => {
                                warn!("Connection {} error: Invalid network or command found, ignore it", address);
                                continue;
                            }
                            _ => {
                                error!("Connection {} error: {}", address, e);
                                return Err(ConnectionError::InvalidDataError);
                            }
                        },
                    };

                    let checksum = calculate_checksum(data);
                    if checksum != header.checksum {
                        error!(
                            "Connection {} error: Checksum mismatch {} vs. {}",
                            address, checksum, header.checksum
                        );
                        return Err(ConnectionError::InvalidDataError);
                    }
                    match header.command {
                        Command::Version => {
                            info!("Connection {}: Received Version message", address);
                            let msg = VersionMessage::decode(&mut data)
                                .map_err(|_| ConnectionError::InvalidDataError)?;

                            other_node_config.version = msg.version;
                            other_node_config.services = msg.services;
                            other_node_config.user_agent = msg.user_agent;
                            other_node_config.start_height = msg.start_height;
                            other_node_config.relay = msg.relay;

                            info!(
                                "Connection {}: Sending Verack message to {}",
                                address, other_node_config.user_agent
                            );
                            let verack_data =
                                compose(network, Command::Verack, VerackMessage {});
                            socket
                                .write_all(&verack_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Verack => {
                            info!("Connection {}: Received Verack message", address);
                            info!("Connection {}: Sending Ping message", address);
                            let ping_data =
                                compose(network, Command::Ping, PingMessage::new());
                            socket
                                .write_all(&ping_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Ping => {
                            let msg = PingMessage::decode(&mut data)
                                .map_err(|_| ConnectionError::InvalidDataError)?;
                            info!(
                                "Connection {}: Received Ping message with nonce {}",
                                address,
                                msg.nonce()
                            );

                            info!("Connection {}: Sending Pong message", address);
                            let pong_data = compose(
                                network,
                                Command::Pong,
                                PongMessage::new(msg.nonce()),
                            );
                            socket
                                .write_all(&pong_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Pong => {
                            let msg = PongMessage::decode(&mut data)
                                .map_err(|_| ConnectionError::InvalidDataError)?;
                            info!(
                                "Connection {}: Received Pong message with nonce {}",
                                address,
                                msg.nonce()
                            );
                            break;
                        }
                    }
                }
            }
        }

        Ok(other_node_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::Ipv4Addr;

    use testcontainers::{
        clients::Cli,
        core::WaitFor,
        GenericImage,
    };

    use crate::p2p::messages::{
        Service,
        Services,
    };

    #[tokio::test]
    #[ignore]
    async fn perform_handshake() {
        let docker = Cli::default();

        let image = GenericImage::new("bitcoin-node", "latest")
            .with_wait_for(WaitFor::seconds(2))
            .with_exposed_port(18444);
        let bitcoin_node = docker.run(image);
        bitcoin_node.start();

        let port = bitcoin_node.get_host_port_ipv4(18444);

        let config = NodeConfig {
            version: 70015,
            services: Services::new(&[Service::Network]),
            user_agent: "test_node".to_string(),
            start_height: 10,
            relay: false,
        };
        let node = Node::new(config);

        let result = node
            .handshake(
                Network::Testnet,
                SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port),
            )
            .await;
        assert!(result.is_ok());
    }
}
