use super::{
    Codec,
    CodecError,
};

use rand::{
    thread_rng,
    Rng,
};

/// Ping message is sent to confirm that the TCP/IP
/// connection is still valid. An error in transmission is
/// presumed to be a closed connection and the address is
/// removed as a current peer.
#[derive(Debug, PartialEq)]
pub struct PingMessage {
    /// Random nonce
    nonce: u64,
}

impl PingMessage {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            nonce: rng.gen::<u64>(),
        }
    }

    /// Gets the random nonce
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Codec for PingMessage {
    const MIN_REQUIRED_LENGTH: usize = 8;

    fn encode(&self) -> Vec<u8> {
        self.nonce.to_le_bytes().to_vec()
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        if data.len() < Self::MIN_REQUIRED_LENGTH {
            return Err(CodecError::InsufficientBytesError);
        }

        let nonce =
            u64::from_le_bytes(data[..std::mem::size_of::<u64>()].try_into().unwrap());
        *data = &data[std::mem::size_of::<u64>()..];

        Ok(Self { nonce })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_PING_MSG: &[u8] = &[0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn encode() {
        let msg = PingMessage { nonce: 15 };
        assert_eq!(msg.encode(), RAW_PING_MSG);
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &RAW_PING_MSG;
        let result = PingMessage::decode(&mut data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().nonce, 15);
        assert!(data.is_empty());
    }

    #[test]
    fn decode_insufficient_bytes() {
        let mut data: &[u8] = &[0x0f, 0x00, 0x00];
        let result = PingMessage::decode(&mut data);

        assert_eq!(result, Err(CodecError::InsufficientBytesError));
        assert!(!data.is_empty());
    }
}
