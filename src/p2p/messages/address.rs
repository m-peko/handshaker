use super::{
    Codec,
    CodecError,
};

use std::net::{
    IpAddr,
    SocketAddr,
};

use crate::p2p::messages::Services;

const IP_ADDRESS_LENGTH: usize = 16;

/// Represents socket address used while exchanging messages in a P2P network.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkAddress {
    /// Features to be enabled for the current connection
    pub services: Services,
    /// IPv4/IPv6 address
    address: [u8; IP_ADDRESS_LENGTH],
    /// Port number in network byte order
    port: u16,
}

impl NetworkAddress {
    pub fn new(services: Services, socket_address: SocketAddr) -> Self {
        let address = match socket_address.ip() {
            IpAddr::V4(ip) => ip.to_ipv6_mapped().octets(),
            IpAddr::V6(ip) => ip.octets(),
        };

        Self {
            services,
            address,
            port: socket_address.port(),
        }
    }

    /// Creates network address with all zero bytes
    pub fn empty() -> Self {
        Self {
            services: Services::empty(),
            address: [0x0_u8; IP_ADDRESS_LENGTH],
            port: 0,
        }
    }

    pub fn address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::from(self.address), self.port)
    }
}

impl Codec for NetworkAddress {
    const MIN_REQUIRED_LENGTH: usize = 26;

    fn encode(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        let mut services_data = self.services.encode();
        data.append(&mut services_data);
        data.extend_from_slice(&self.address);
        data.extend_from_slice(&self.port.to_be_bytes());
        data
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        if data.len() < Self::MIN_REQUIRED_LENGTH {
            return Err(CodecError::InsufficientBytesError);
        }

        let services = Services::decode(data)?;

        let address: [u8; IP_ADDRESS_LENGTH] =
            data[..IP_ADDRESS_LENGTH].try_into().unwrap();
        *data = &data[IP_ADDRESS_LENGTH..];

        let port =
            u16::from_be_bytes(data[..std::mem::size_of::<u16>()].try_into().unwrap());
        *data = &data[std::mem::size_of::<u16>()..];

        println!("{}", data.len());

        Ok(Self {
            services,
            address,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    use crate::p2p::messages::Service;

    #[rustfmt::skip]
    mod unformatted {
        pub const RAW_NET_ADDRESS: &[u8] = &[
            // Services
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // IP address
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x0a,
            0x00, 0x00, 0x01,
            // Port
            0x20, 0x8d,
        ];
    }

    use unformatted::*;

    #[test]
    fn encode() {
        let services = Services::new(&[Service::Network, Service::Bloom]);
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 8333);

        let net_address = NetworkAddress::new(services, socket);
        assert_eq!(net_address.encode(), RAW_NET_ADDRESS);
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &RAW_NET_ADDRESS;
        let result = NetworkAddress::decode(&mut data);
        assert!(result.is_ok());

        let net_address = result.unwrap();
        assert_eq!(
            net_address.services.enabled(),
            [Service::Network, Service::Bloom]
        );

        let socket_address = net_address.address();
        assert_eq!(
            socket_address.ip(),
            Ipv4Addr::new(10, 0, 0, 1).to_ipv6_mapped()
        );
        assert_eq!(socket_address.port(), 8333);
        assert!(data.is_empty());
    }

    #[test]
    fn decode_insufficient_bytes() {
        let mut data: &[u8] = &[0x0f, 0x00, 0x00];
        let result = NetworkAddress::decode(&mut data);

        assert_eq!(result, Err(CodecError::InsufficientBytesError));
        assert!(!data.is_empty());
    }
}

