use std::fmt;
use std::fmt::Debug;
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

// Implement `fmt::Display` for `RequestError`
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::CantSetHeadersAfterRequestSent => write!(f, "Cannot set headers after request sent"),
            RequestError::CantResolveUrl => write!(f, "Cannot resolve given URL"),
            RequestError::ConnectionTimeout => write!(f, "Connection timed out"),
            RequestError::MalformedUrl => write!(f, "Given URL is malformed"),
            RequestError::AlreadySent => write!(f, "Request already sent"),
            RequestError::ConnectionError(err) => write!(f, "Connection error: {}", err),
        }
    }
}

// Implement `std::error::Error` for `RequestError`
impl std::error::Error for RequestError {}

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

impl From<anyhow::Error> for RequestError {
    fn from(value: anyhow::Error) -> Self {
        RequestError::ConnectionError(value.to_string())
    }
}