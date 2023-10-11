use clap::Parser;
use std::{
    fmt::{
        Display,
        Formatter,
    },
    net::SocketAddrV4,
    num::ParseIntError,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(
        num_args = 1..,
        required = true,
        value_parser = parse_socket_address,
        value_delimiter = ' ',
        help = "P2P node IPv4 socket addresses to perform handshakes with"
    )]
    pub addresses: Vec<SocketAddrV4>,

    #[arg(
        short,
        long,
        default_value = "1000",
        value_parser = parse_timeout,
        help = "Maximum time per message in milliseconds"
    )]
    pub timeout: Duration,
}

#[derive(Debug, PartialEq)]
enum SockerAddrV4Error {
    MissingAddrError,
    MissingPortError,
    InvalidAddrError,
    InvalidAddrComponentRangeError,
    InvalidPortRangeError,
}

impl Display for SockerAddrV4Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SockerAddrV4Error::MissingAddrError => {
                write!(f, "IPv4 address not specified")
            }
            SockerAddrV4Error::MissingPortError => write!(f, "Port not specified"),
            SockerAddrV4Error::InvalidAddrError => write!(
                f,
                "IPv4 address should consist of \
                four decimal numbers, each ranging from 0 to 255"
            ),
            SockerAddrV4Error::InvalidAddrComponentRangeError => {
                write!(f, "IPv4 address component should range from 0 to 255")
            }
            SockerAddrV4Error::InvalidPortRangeError => {
                write!(f, "Port should range from 0 to 65536")
            }
        }
    }
}

impl std::error::Error for SockerAddrV4Error {}

fn parse_socket_address(socket_addr: &str) -> Result<SocketAddrV4, SockerAddrV4Error> {
    match socket_addr.parse() {
        Ok(v) => Ok(v),
        Err(_) => {
            // Since AddrParseError is not that verbose, implement
            // our own address validation to get more verbose CLI error
            let (addr, port) = socket_addr.split_once(':').unwrap_or((socket_addr, ""));
            if addr.is_empty() {
                return Err(SockerAddrV4Error::MissingAddrError);
            }

            let components = addr.split('.');
            if components.clone().count() != 4 {
                return Err(SockerAddrV4Error::InvalidAddrError);
            }

            for c in components {
                if let Err(_) = c.parse::<u8>() {
                    return Err(SockerAddrV4Error::InvalidAddrComponentRangeError);
                }
            }

            if port.is_empty() {
                return Err(SockerAddrV4Error::MissingPortError);
            }

            if let Err(_) = port.parse::<u16>() {
                return Err(SockerAddrV4Error::InvalidPortRangeError);
            }

            debug_assert!(false);
            Err(SockerAddrV4Error::InvalidAddrError)
        }
    }
}

fn parse_timeout(timeout: &str) -> Result<Duration, ParseIntError> {
    let millis = timeout.parse()?;
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn validate_socket_address_arg() {
        assert_eq!(
            parse_socket_address("random input"),
            Err(SockerAddrV4Error::InvalidAddrError)
        );
        assert_eq!(
            parse_socket_address(":3000"),
            Err(SockerAddrV4Error::MissingAddrError)
        );
        assert_eq!(
            parse_socket_address("127.0.0.1"),
            Err(SockerAddrV4Error::MissingPortError)
        );
        assert_eq!(
            parse_socket_address("127.0.0.1:"),
            Err(SockerAddrV4Error::MissingPortError)
        );
        assert_eq!(
            parse_socket_address("127.0.0:3000"),
            Err(SockerAddrV4Error::InvalidAddrError)
        );
        assert_eq!(
            parse_socket_address("127.0.0.266:3000"),
            Err(SockerAddrV4Error::InvalidAddrComponentRangeError)
        );
        assert_eq!(
            parse_socket_address("127.0.0.1:70000"),
            Err(SockerAddrV4Error::InvalidPortRangeError)
        );
        assert_eq!(
            parse_socket_address("127.0.0.1:3000"),
            Ok(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3000))
        );
    }

    #[test]
    fn validate_timeout_arg() {
        {
            let result = parse_timeout("abc");
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "invalid digit found in string"
            );
        }
        assert_eq!(parse_timeout("1000"), Ok(Duration::from_millis(1000)));
    }
}
