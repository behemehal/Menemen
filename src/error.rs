use std::net::TcpStream;

#[cfg(feature = "https")]
use native_tls::{Error, HandshakeError};

/// List of request errors
#[derive(Clone, Debug)]
pub enum RequestError {
    /// Cannot set headers after they sent
    CantSetHeadersAfterRequestSent,
    /// Cannot resolve given url
    CantResolveUrl,
    /// Connection timed out
    ConnectionTimeout,
    /// Given url is not correct
    MalformedUrl,
    /// Request already sent
    AlreadySent,
    /// Connection error occurred with string
    ConnectionError(String),
}

impl From<std::io::Error> for RequestError {
    fn from(error: std::io::Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(feature = "https")]
impl From<HandshakeError<TcpStream>> for RequestError {
    fn from(error: HandshakeError<TcpStream>) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(feature = "https")]
impl From<anyhow::Error> for RequestError {
    fn from(error: anyhow::Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(feature = "https")]
impl From<Error> for RequestError {
    fn from(error: Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

impl From<String> for RequestError {
    fn from(error: String) -> Self {
        RequestError::ConnectionError(error)
    }
}

impl From<&str> for RequestError {
    fn from(error: &str) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}
