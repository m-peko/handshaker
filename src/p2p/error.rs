use std::fmt::{
    Display,
    Formatter,
};

#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    ConnectionHangUp,
    ConnectionRefusedError,
    InvalidDataError,
    IOError,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::ConnectionHangUp => {
                write!(f, "Connection hang up")
            }
            ConnectionError::ConnectionRefusedError => {
                write!(f, "Connection to provided address refused")
            }
            ConnectionError::InvalidDataError => {
                write!(f, "Invalid data received")
            }
            ConnectionError::IOError => {
                write!(f, "IO error occurred during connection")
            }
        }
    }
}

impl std::error::Error for ConnectionError {}
