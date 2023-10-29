use super::{
    Codec,
    CodecError,
};

/// Verack message is sent in response to Version message.
/// It consists of only a message header with the command
/// string "verack".
#[derive(Debug)]
pub struct VerackMessage {}

impl Codec for VerackMessage {
    fn encode(&self) -> Vec<u8> {
        Vec::<u8>::new()
    }

    fn decode(data: &mut &[u8]) -> Result<Self, CodecError> {
        Ok(Self {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        let msg = VerackMessage {};
        assert!(msg.encode().is_empty());
    }

    #[test]
    fn decode() {
        let mut data: &[u8] = &[];
        let mut result = VerackMessage::decode(&mut data);

        assert!(result.is_ok());
        assert!(data.is_empty());

        data = &[0xff, 0x01, 0x00];
        result = VerackMessage::decode(&mut data);

        assert!(result.is_ok());
        assert!(!data.is_empty());
    }
}
