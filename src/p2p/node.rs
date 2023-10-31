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
        Command,
        MessageHeader,
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
        write!(f, "version: {}", self.version)?;
        write!(f, "services: {}", self.services)?;
        write!(f, "user agent: {}", self.user_agent)?;
        write!(f, "start height: {}", self.start_height)?;
        write!(f, "relay: {}", self.relay)
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
        address: SocketAddrV4,
    ) -> Result<NodeConfig, ConnectionError> {
        let mut other_node_config: NodeConfig = Default::default();

        let mut socket = TcpStream::connect(address)
            .await
            .map_err(|_| ConnectionError::ConnectionRefusedError)?;

        let version_data = compose(
            Command::Version,
            VersionMessage::new(SocketAddr::from(address), &self.config),
        );
        socket
            .write_all(&version_data[..])
            .await
            .map_err(|_| ConnectionError::IOError)?;

        loop {
            let mut buffer = [0; 1024];
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
                        Err(e) => {
                            error!("Connection error: {}", e);
                            return Err(ConnectionError::InvalidDataError);
                        }
                    };

                    let checksum = calculate_checksum(data);
                    if checksum != header.checksum {
                        error!(
                            "Connection error: Checksum mismatch {} vs. {}",
                            checksum, header.checksum
                        );
                        return Err(ConnectionError::InvalidDataError);
                    }

                    match header.command {
                        Command::Version => {
                            let msg = VersionMessage::decode(&mut data)
                                .map_err(|_| ConnectionError::InvalidDataError)?;

                            info!(
                                "Connection: Received Version message from {}",
                                msg.user_agent
                            );

                            other_node_config.version = msg.version;
                            other_node_config.services = msg.services;
                            other_node_config.user_agent = msg.user_agent;
                            other_node_config.start_height = msg.start_height;
                            other_node_config.relay = msg.relay;

                            // send Verack message
                            let verack_data = compose(Command::Verack, VerackMessage {});
                            socket
                                .write_all(&verack_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Verack => {
                            info!(
                                "Connection: Received Verack, sending out Ping message"
                            );

                            // send Ping message
                            let ping_data = compose(Command::Ping, PingMessage::new());
                            socket
                                .write_all(&ping_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Ping => {
                            info!("Connection: Received Ping, sending out Pong message");

                            let msg = PingMessage::decode(&mut data)
                                .map_err(|_| ConnectionError::InvalidDataError)?;

                            // send Pong message
                            let pong_data =
                                compose(Command::Pong, PongMessage::new(msg.nonce()));
                            socket
                                .write_all(&pong_data[..])
                                .await
                                .map_err(|_| ConnectionError::IOError)?;
                        }
                        Command::Pong => {
                            info!("Connection: Received Pong, handshake successfully performed");
                            break;
                        }
                    }
                }
            }
        }

        Ok(other_node_config)
    }
}
