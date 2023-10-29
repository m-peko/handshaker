use super::{
    Codec,
    CodecError,
    ReadBytes,
};

use strum::{
    EnumIter,
    IntoEnumIterator,
};

/// Represents services nodes can provide to the network.
/// Associated values represent bit masks used to check if
/// the specific service bit is set in the protocol message.
#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
#[repr(u64)]
pub enum Service {
    Network = 0x00_00_00_00_00_00_00_01,
    Getutx = 0x00_00_00_00_00_00_00_02,
    Bloom = 0x00_00_00_00_00_00_00_04,
    Witness = 0x00_00_00_00_00_00_00_08,
    Xthin = 0x00_00_00_00_00_00_00_10,
    CompactFilters = 0x00_00_00_00_00_00_00_40,
    NetworkLimited = 0x00_00_00_00_00_00_04_00,
}

impl Service {
    fn as_u64(self) -> u64 {
        self as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Services {
    services: u64,
}

impl Services {
    pub fn new(services: &[Service]) -> Self {
        let mut data: u64 = 0;
        for s in services {
            data |= s.as_u64();
        }
        Self { services: data }
    }

    /// Creates services with all zero bytes
    pub fn empty() -> Self {
        Self { services: 0 }
    }

    /// Gets enabled services
    pub fn enabled(&self) -> Vec<Service> {
        let mut services = Vec::new();
        for s in Service::iter() {
            if self.services & s.as_u64() != 0 {
                services.push(s);
            }
        }
        services
    }
}

impl From<u64> for Services {
    fn from(services: u64) -> Self {
        Self { services }
    }
}

impl Codec for Services {
    fn encode(&self) -> Vec<u8> {
        self.services.to_le_bytes().to_vec()
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        let services = data
            .read_le::<u64>()
            .ok_or(CodecError::InsufficientBytesError)?;
        Ok(Self { services })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_SERVICES: &[u8] = &[0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn new() {
        let mut raw_services = vec![Service::Network, Service::Witness];
        let mut services = Services::new(&raw_services[..]);

        assert_eq!(services, 0x00_00_00_00_00_00_00_09.into());
        assert_eq!(services.enabled(), raw_services);

        raw_services.push(Service::CompactFilters);
        raw_services.push(Service::NetworkLimited);

        services = Services::new(&raw_services[..]);

        assert_eq!(services, 0x00_00_00_00_00_00_04_49.into());
        assert_eq!(services.enabled(), raw_services);
    }

    #[test]
    fn encode() {
        let services =
            Services::new(&[Service::Network, Service::Bloom, Service::Witness]);
        assert_eq!(services.encode(), RAW_SERVICES);
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &RAW_SERVICES;
        let result = Services::decode(&mut data);

        assert!(result.is_ok());
        assert!(data.is_empty());

        let services = result.unwrap();
        let expected_raw_services = [Service::Network, Service::Bloom, Service::Witness];

        assert_eq!(services.enabled(), expected_raw_services);
    }

    #[test]
    fn decode_insufficient_bytes() {
        let mut data: &[u8] = &[0x0d, 0x00, 0x00, 0x00];
        let result = Services::decode(&mut data);

        assert_eq!(result, Err(CodecError::InsufficientBytesError));
        assert!(!data.is_empty());
    }
}
