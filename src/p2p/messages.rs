use std::fmt::{
    Display,
    Formatter,
};

use sha2::{
    Digest,
    Sha256,
};
use strum::{
    EnumIter,
    IntoEnumIterator,
};

pub mod address;
pub mod ping;
pub mod pong;
pub mod services;
pub mod verack;
pub mod version;

pub use address::*;
pub use ping::*;
pub use pong::*;
pub use services::*;
pub use verack::*;
pub use version::*;

trait FromBytes {
    fn from_be_bytes(bytes: &[u8]) -> Self;
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

macro_rules! impl_from_bytes {
    ($t:ident) => {
        impl FromBytes for $t {
            fn from_be_bytes(bytes: &[u8]) -> Self {
                $t::from_be_bytes(bytes[..std::mem::size_of::<$t>()].try_into().unwrap())
            }

            fn from_le_bytes(bytes: &[u8]) -> Self {
                $t::from_le_bytes(bytes[..std::mem::size_of::<$t>()].try_into().unwrap())
            }
        }
    };
}

impl_from_bytes!(u8);
impl_from_bytes!(u16);
impl_from_bytes!(u32);
impl_from_bytes!(u64);
impl_from_bytes!(i16);
impl_from_bytes!(i32);
impl_from_bytes!(i64);

trait ReadBytes {
    fn read_le<T: FromBytes>(&mut self) -> Option<T>;
    fn read_be<T: FromBytes>(&mut self) -> Option<T>;
    fn read_fixed<const N: usize>(&mut self) -> Option<[u8; N]>;
    fn read_slice(&mut self, n: usize) -> Option<&[u8]>;
}

impl ReadBytes for &[u8] {
    fn read_le<T: FromBytes>(&mut self) -> Option<T> {
        let len = std::mem::size_of::<T>();
        if self.len() < len {
            return None;
        }

        let value = T::from_le_bytes(self[0..len].try_into().unwrap());
        *self = &self[len..];
        Some(value)
    }

    fn read_be<T: FromBytes>(&mut self) -> Option<T> {
        let len = std::mem::size_of::<T>();
        if self.len() < len {
            return None;
        }

        let value = T::from_be_bytes(self[0..len].try_into().unwrap());
        *self = &self[len..];
        Some(value)
    }

    fn read_fixed<const N: usize>(&mut self) -> Option<[u8; N]> {
        if self.len() < N {
            return None;
        }

        let value = self[..N].try_into().unwrap();
        *self = &self[N..];
        Some(value)
    }

    fn read_slice(&mut self, n: usize) -> Option<&[u8]> {
        if self.len() < n {
            return None;
        }

        let value = &self[..n];
        *self = &self[n..];
        Some(value)
    }
}

pub trait Codec {
    /// Encodes an object of a specific type into a stream of bytes in
    /// network byte order, i.e big-endian.
    fn encode(&self) -> Vec<u8>;

    /// Decodes stream of bytes in network byte order, i.e. big-endian,
    /// into an object.
    fn decode(data: &mut &[u8]) -> Result<Self, CodecError>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq)]
pub enum CodecError {
    InvalidBytesError,
    InsufficientBytesError,
}

impl Display for CodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::InvalidBytesError => {
                write!(f, "Invalid bytes provided during decoding")
            }
            CodecError::InsufficientBytesError => {
                write!(f, "Insufficient amount of bytes provided during decoding")
            }
        }
    }
}

impl std::error::Error for CodecError {}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
#[repr(u32)]
pub enum Network {
    Main = 0xd9_b4_be_f9,
    Testnet = 0xda_b5_bf_fa,
    Testnet3 = 0x07_09_11_0b,
    Signet = 0x40_cf_03_0a,
    Namecoin = 0xfe_b4_be_f9,
}

impl Into<u32> for Network {
    fn into(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for Network {
    type Error = &'static str;

    fn try_from(data: u32) -> Result<Self, Self::Error> {
        for n in Network::iter() {
            if data == n.into() {
                return Ok(n);
            }
        }
        return Err("Unknown network identifier");
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Command {
    Version,
    Verack,
    Ping,
    Pong,
}

impl Command {
    const REQUIRED_LENGTH: usize = 12;

    fn to_bytes(self) -> &'static [u8; Self::REQUIRED_LENGTH] {
        match self {
            Command::Version => b"version\0\0\0\0\0",
            Command::Verack => b"verack\0\0\0\0\0\0",
            Command::Ping => b"ping\0\0\0\0\0\0\0\0",
            Command::Pong => b"pong\0\0\0\0\0\0\0\0",
        }
    }
}

impl TryFrom<&[u8; Command::REQUIRED_LENGTH]> for Command {
    type Error = &'static str;

    fn try_from(data: &[u8; Self::REQUIRED_LENGTH]) -> Result<Self, Self::Error> {
        for c in Command::iter() {
            if c.to_bytes() == data {
                return Ok(c);
            }
        }
        Err("Unknown command")
    }
}

#[derive(Debug)]
pub struct MessageHeader {
    /// Identifier of the origin network
    pub network: Network,
    /// Identifier of the packet content
    pub command: Command,
    /// Payload length in number of bytes
    pub length: u32,
    /// First 4 bytes of sha256(sha256(payload))
    pub checksum: u32,
}

impl Codec for MessageHeader {
    fn encode(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&(self.network as u32).to_le_bytes());
        data.extend_from_slice(self.command.to_bytes());
        data.extend_from_slice(&self.length.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());
        data
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        let network = Network::try_from(
            data.read_le::<u32>()
                .ok_or(CodecError::InsufficientBytesError)?,
        )
        .map_err(|_| CodecError::InvalidBytesError)?;

        let command = Command::try_from(
            &data
                .read_fixed::<{ Command::REQUIRED_LENGTH }>()
                .ok_or(CodecError::InsufficientBytesError)?,
        )
        .map_err(|_| CodecError::InvalidBytesError)?;

        let length = data
            .read_le::<u32>()
            .ok_or(CodecError::InsufficientBytesError)?;
        let checksum = data
            .read_le::<u32>()
            .ok_or(CodecError::InsufficientBytesError)?;

        Ok(Self {
            network,
            command,
            length,
            checksum,
        })
    }
}

pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut result = hasher.finalize_reset();

    hasher.update(result);
    result = hasher.finalize();

    u32::from_le_bytes(result[..std::mem::size_of::<u32>()].try_into().unwrap())
}

pub fn compose(command: Command, payload: impl Codec) -> Vec<u8> {
    let payload_data = payload.encode();
    let header = MessageHeader {
        network: Network::Main,
        command,
        length: payload_data.len() as u32,
        checksum: calculate_checksum(&payload_data[..]),
    }
    .encode();

    let mut data = Vec::<u8>::new();
    data.extend(header);
    data.extend(payload_data);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    mod unformatted {
        pub const RAW_HEADER: &[u8] = &[
            // Magic bytes
            0xf9, 0xbe, 0xb4, 0xd9,
            // Version command
            0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Payload length
            0x64, 0x00, 0x00, 0x00,
            // Payload checksum
            0x35, 0x8d, 0x49, 0x32,
        ];
    }

    use unformatted::*;

    #[test]
    fn encode() {
        let header = MessageHeader {
            network: Network::Main,
            command: Command::Version,
            length: 100,
            checksum: 0x32498d35,
        };
        assert_eq!(header.encode(), RAW_HEADER);
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &RAW_HEADER;
        let result = MessageHeader::decode(&mut data);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.network, Network::Main);
        assert_eq!(header.command, Command::Version);
        assert_eq!(header.length, 100);
        assert_eq!(header.checksum, 0x32498d35);
    }

    #[test]
    fn checksum() {
        let checksum = calculate_checksum(&[]);
        assert_eq!(checksum, 0xe2e0f65d);
    }
}
