use super::{
    Codec,
    CodecError,
};

/// Pong message is sent in response to a Ping message.
#[derive(Debug, PartialEq)]
pub struct PongMessage {
    // Nonce from Ping message
    nonce: u64,
}

impl PongMessage {
    pub fn new(nonce: u64) -> Self {
        Self { nonce }
    }

    /// Gets the nonce from Ping message
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Codec for PongMessage {
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

    const RAW_PONG_MSG: &[u8] = &[0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn encode() {
        let msg = PongMessage { nonce: 15 };
        assert_eq!(msg.encode(), RAW_PONG_MSG);
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &RAW_PONG_MSG;
        let result = PongMessage::decode(&mut data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().nonce, 15);
        assert!(data.is_empty());
    }

    #[test]
    fn decode_insufficient_bytes() {
        let mut data: &[u8] = &[0x0f, 0x00, 0x00];
        let result = PongMessage::decode(&mut data);

        assert_eq!(result, Err(CodecError::InsufficientBytesError));
        assert!(!data.is_empty());
    }
}
