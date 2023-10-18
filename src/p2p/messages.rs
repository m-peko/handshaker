use std::fmt::{
    Display,
    Formatter,
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

pub trait Codec {
    /// Minimum length in bytes needed to decode stream into an object.
    const MIN_REQUIRED_LENGTH: usize;

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
    InsufficientBytesError,
}

impl Display for CodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::InsufficientBytesError => {
                write!(f, "Insufficient amount of bytes provided during decoding")
            }
        }
    }
}

impl std::error::Error for CodecError {}

#[derive(Clone, Copy, Debug, EnumIter)]
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

#[derive(Clone, Copy, Debug, EnumIter)]
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

impl TryFrom<&[u8]> for Command {
    type Error = &'static str;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::REQUIRED_LENGTH {
            return Err("Too short command");
        }

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
    const MIN_REQUIRED_LENGTH: usize = 24;

    fn encode(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&(self.network as u32).to_le_bytes());
        data.extend_from_slice(self.command.to_bytes());
        data.extend_from_slice(&self.length.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());
        data
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        if data.len() < Self::MIN_REQUIRED_LENGTH {
            return Err(CodecError::InsufficientBytesError);
        }

        let network = Network::try_from(u32::from_le_bytes(
            data[..std::mem::size_of::<u32>()].try_into().unwrap(),
        ))
        .unwrap();
        *data = &data[std::mem::size_of::<u32>()..];

        let command = Command::try_from(*data).unwrap();
        *data = &data[12..];

        let length =
            u32::from_le_bytes(data[..std::mem::size_of::<u32>()].try_into().unwrap());
        *data = &data[std::mem::size_of::<u32>()..];

        let checksum =
            u32::from_le_bytes(data[..std::mem::size_of::<u32>()].try_into().unwrap());
        *data = &data[std::mem::size_of::<u32>()..];

        Ok(Self {
            network,
            command,
            length,
            checksum,
        })
    }
}
