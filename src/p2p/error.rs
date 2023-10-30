use std::fmt::{
    Display,
    Formatter,
};

#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    InvalidDataError,
    IOError,
    ConnectionHangUp,
    ConnectionRefusedError,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::InvalidDataError => {
                write!(f, "Invalid data received")
            }
            ConnectionError::IOError => {
                write!(f, "IO error occurred during connection")
            }
            ConnectionError::ConnectionHangUp => {
                write!(f, "Connection hang up")
            }
            ConnectionError::ConnectionRefusedError => {
                write!(f, "Connection to provided address refused")
            }
        }
    }
}

impl std::error::Error for ConnectionError {}
