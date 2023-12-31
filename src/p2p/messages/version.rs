use super::{
    Codec,
    CodecError,
    ReadBytes,
};

use crate::p2p::{
    messages::{
        NetworkAddress,
        Services,
    },
    NodeConfig,
};

use std::{
    net::SocketAddr,
    time::{
        Duration,
        SystemTime,
        UNIX_EPOCH,
    },
};

use rand::{
    thread_rng,
    Rng,
};

#[derive(Debug)]
pub struct VersionMessage {
    /// Protocol version used by the node
    pub version: i32,
    /// Features to be enabled for the current connection
    pub services: Services,
    /// Standard UNIX timestamp in seconds
    timestamp: i64,
    /// Network address of the node receiving this message
    receiver: NetworkAddress,
    /// Network address of the node sending this message (can be ignored)
    sender: NetworkAddress,
    /// Random nonce used to detect connections to self
    nonce: u64,
    /// User agent
    pub user_agent: String,
    /// Last block received by the emitting node
    pub start_height: i32,
    /// Whether the remote peer should announce relayed transactions or not
    pub relay: bool,
}

impl VersionMessage {
    pub fn new(receiver: SocketAddr, config: &NodeConfig) -> Self {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(v) => v.as_secs() as i64,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let mut rng = thread_rng();

        Self {
            version: config.version,
            services: config.services,
            timestamp,
            receiver: NetworkAddress::new(config.services, receiver),
            sender: NetworkAddress::empty(),
            nonce: rng.gen::<u64>(),
            user_agent: config.user_agent.clone(),
            start_height: config.start_height,
            relay: config.relay,
        }
    }

    /// Gets the UNIX timestamp
    #[allow(dead_code)]
    pub fn timestamp(&self) -> Duration {
        Duration::from_secs(self.timestamp.try_into().unwrap())
    }

    /// Gets the receiver's node address
    #[allow(dead_code)]
    pub fn receiver(&self) -> &NetworkAddress {
        &self.receiver
    }

    /// Gets the random nonce
    #[allow(dead_code)]
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Codec for VersionMessage {
    fn encode(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&self.version.to_le_bytes());

        let services_data = self.services.encode();
        data.extend(services_data);
        data.extend_from_slice(&self.timestamp.to_le_bytes());

        let to_net_address_data = self.receiver.encode();
        data.extend(to_net_address_data);

        if self.version < 106 {
            return data;
        }

        let from_net_address_data = self.sender.encode();
        data.extend(from_net_address_data);
        data.extend_from_slice(&self.nonce.to_be_bytes());

        // Encode user agent (byte indicating field length + string)
        data.push(self.user_agent.len() as u8);
        if !self.user_agent.is_empty() {
            data.extend_from_slice(&self.user_agent.as_bytes());
        }

        data.extend_from_slice(&self.start_height.to_le_bytes());

        if self.version < 70001 {
            return data;
        }

        data.push(self.relay as u8);
        data
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        let version = data
            .read_le::<i32>()
            .ok_or(CodecError::InsufficientBytesError)?;
        let services = Services::decode(data)?;
        let timestamp = data
            .read_le::<i64>()
            .ok_or(CodecError::InsufficientBytesError)?;
        let receiver = NetworkAddress::decode(data)?;

        if version < 106 {
            return Ok(Self {
                version,
                services,
                timestamp,
                receiver,
                sender: NetworkAddress::empty(),
                nonce: 0,
                user_agent: String::new(),
                start_height: 0,
                relay: false,
            });
        }

        let sender = NetworkAddress::decode(data)?;
        let nonce = data
            .read_be::<u64>()
            .ok_or(CodecError::InsufficientBytesError)?;
        let user_agent_length = data
            .read_be::<u8>()
            .ok_or(CodecError::InsufficientBytesError)?;

        let mut user_agent = String::new();
        if user_agent_length != 0 {
            let user_agent_data = data
                .read_slice(user_agent_length as usize)
                .ok_or(CodecError::InsufficientBytesError)?;
            user_agent = std::str::from_utf8(user_agent_data).unwrap().to_string();
        }

        let start_height = data
            .read_le::<i32>()
            .ok_or(CodecError::InsufficientBytesError)?;

        if version < 70001 {
            return Ok(Self {
                version,
                services,
                timestamp,
                receiver,
                sender,
                nonce,
                user_agent,
                start_height,
                relay: false,
            });
        }

        let relay = data
            .read_le::<u8>()
            .ok_or(CodecError::InsufficientBytesError)?
            == 0x01;

        Ok(Self {
            version,
            services,
            timestamp,
            receiver,
            sender,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip] 
    mod unformatted {
        pub const RAW_VERSION_MSG_LT_106: &[u8] = &[
            // Version
            0x64, 0x00, 0x00, 0x00,
            // Services
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Timestamp
            0xe6, 0x15, 0x10, 0x4d, 0x00, 0x00, 0x00, 0x00,
            // Receiver's network address
                // Services
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // IP address
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xff, 0xff, 0x0a, 0x00, 0x00, 0x01,
                // Port
                0x20, 0x8d,
        ];

        pub const RAW_VERSION_MSG_LT_70001: &[u8] = &[
            // Version
            0x70, 0x11, 0x01, 0x00,
            // Services
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Timestamp
            0xe6, 0x15, 0x10, 0x4d, 0x00, 0x00, 0x00, 0x00,
            // Receiver's network address
                // Services
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // IP address
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xff, 0xff, 0x0a, 0x00, 0x00, 0x01,
                // Port
                0x20, 0x8d,
            // Sender's network address
                // Services
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // IP address
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // Port
                0x00, 0x00,
            // Nonce
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x39,
            // User agent
            0x0f, 0x2f, 0x53, 0x61, 0x74, 0x6f, 0x73, 0x68,
            0x69, 0x3a, 0x30, 0x2e, 0x37, 0x2e, 0x32, 0x2f,
            // Start height
            0xc0, 0x3e, 0x03, 0x00,
        ];

        pub const RAW_VERSION_MSG_GE_70001: &[u8] = &[
            // Version
            0x71, 0x11, 0x01, 0x00,
            // Services
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Timestamp
            0xe6, 0x15, 0x10, 0x4d, 0x00, 0x00, 0x00, 0x00,
            // Receiver's network address
                // Services
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // IP address
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0xff, 0xff, 0x0a, 0x00, 0x00, 0x01,
                // Port
                0x20, 0x8d,
            // Sender's network address
                // Services
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // IP address
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // Port
                0x00, 0x00,
            // Nonce
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x39,
            // User agent
            0x00,
            // Start height
            0xc0, 0x3e, 0x03, 0x00,
            // Relay
            0x01,
        ];
    }

    use super::*;
    use unformatted::*;

    use std::net::{
        IpAddr,
        Ipv4Addr,
        SocketAddr,
    };

    use chrono::prelude::*;
    use lazy_static::lazy_static;

    use crate::p2p::messages::Service;

    lazy_static! {
        static ref SERVICES: Services = Services::new(&[Service::Network]);
        static ref SOCKET: SocketAddr =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 8333);
        static ref RECEIVER: NetworkAddress = NetworkAddress::new(*SERVICES, *SOCKET);
        static ref TIMESTAMP: i64 = DateTime::parse_from_str(
            "2010-12-20 21:50:14 -05:00",
            "%Y-%m-%d %H:%M:%S %z"
        )
        .unwrap()
        .timestamp();
    }

    #[test]
    fn encode_version_lt_106() {
        let msg = VersionMessage {
            version: 100,
            services: *SERVICES,
            timestamp: *TIMESTAMP,
            receiver: *RECEIVER,
            // below fields are not used during encoding when version is < 106
            sender: NetworkAddress::empty(),
            nonce: 0,
            user_agent: String::new(),
            start_height: 0,
            relay: false,
        };
        assert_eq!(msg.encode(), RAW_VERSION_MSG_LT_106);
    }

    #[test]
    fn encode_version_lt_70001() {
        let msg = VersionMessage {
            version: 70000,
            services: *SERVICES,
            timestamp: *TIMESTAMP,
            receiver: *RECEIVER,
            sender: NetworkAddress::empty(),
            nonce: 12345,
            user_agent: String::from("/Satoshi:0.7.2/"),
            start_height: 212672,
            // below fields are not used during encoding when version is < 70001
            relay: false,
        };
        assert_eq!(msg.encode(), RAW_VERSION_MSG_LT_70001);
    }

    #[test]
    fn encode_version_ge_70001() {
        let msg = VersionMessage {
            version: 70001,
            services: *SERVICES,
            timestamp: *TIMESTAMP,
            receiver: *RECEIVER,
            sender: NetworkAddress::empty(),
            nonce: 12345,
            user_agent: String::new(),
            start_height: 212672,
            relay: true,
        };
        assert_eq!(msg.encode(), RAW_VERSION_MSG_GE_70001);
    }

    #[test]
    fn decode_version_lt_106() {
        let mut data: &[u8] = &RAW_VERSION_MSG_LT_106;
        let result = VersionMessage::decode(&mut data);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.version, 100);
        assert_eq!(msg.services, *SERVICES);
        assert_eq!(msg.timestamp().as_secs() as i64, *TIMESTAMP);
        assert_eq!(*msg.receiver(), *RECEIVER);
    }

    #[test]
    fn decode_version_lt_70001() {
        let mut data: &[u8] = &RAW_VERSION_MSG_LT_70001;
        let result = VersionMessage::decode(&mut data);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.version, 70000);
        assert_eq!(msg.services, *SERVICES);
        assert_eq!(msg.timestamp().as_secs() as i64, *TIMESTAMP);
        assert_eq!(*msg.receiver(), *RECEIVER);
        assert_eq!(msg.nonce(), 12345);
        assert_eq!(msg.user_agent, "/Satoshi:0.7.2/");
        assert_eq!(msg.start_height, 212672);
    }

    #[test]
    fn decode_version_ge_70001() {
        let mut data: &[u8] = &RAW_VERSION_MSG_GE_70001;
        let result = VersionMessage::decode(&mut data);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.version, 70001);
        assert_eq!(msg.services, *SERVICES);
        assert_eq!(msg.timestamp().as_secs() as i64, *TIMESTAMP);
        assert_eq!(*msg.receiver(), *RECEIVER);
        assert_eq!(msg.nonce(), 12345);
        assert_eq!(msg.user_agent, "");
        assert_eq!(msg.start_height, 212672);
        assert_eq!(msg.relay, true);
    }
}
