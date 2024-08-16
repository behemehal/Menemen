use std::fmt;
use std::fmt::Debug;

#[cfg(all(not(feature = "async"), feature = "https"))]
use std::net::TcpStream;

#[cfg(all(feature = "https", not(feature = "async")))]
use native_tls;

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
    /// TLS feature is not enabled
    TlsNotEnabled,
}

// Implement `fmt::Display` for `RequestError`
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::CantSetHeadersAfterRequestSent => {
                write!(f, "Cannot set headers after request sent")
            }
            RequestError::CantResolveUrl => write!(f, "Cannot resolve given URL"),
            RequestError::ConnectionTimeout => write!(f, "Connection timed out"),
            RequestError::MalformedUrl => write!(f, "Given URL is malformed"),
            RequestError::AlreadySent => write!(f, "Request already sent"),
            RequestError::ConnectionError(err) => write!(f, "Connection error: {}", err),
            RequestError::TlsNotEnabled => write!(f, "TLS feature is not enabled, enable it to use HTTPS"),
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

#[cfg(all(feature = "https", not(feature = "async")))]
impl From<native_tls::HandshakeError<TcpStream>> for RequestError {
    fn from(error: native_tls::HandshakeError<TcpStream>) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

impl From<anyhow::Error> for RequestError {
    fn from(error: anyhow::Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(feature = "https-async")]
impl From<native_tls::Error> for RequestError {
    fn from(error: native_tls::Error) -> Self {
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
