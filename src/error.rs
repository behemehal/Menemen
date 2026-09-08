use std::fmt;
use std::fmt::Debug;

#[cfg(all(feature = "https", feature = "blocking"))]
use native_tls;

#[cfg(feature = "json")]
use serde_json::Error as JsonError;

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
    /// File error, provided path string does not exist
    FileError(String),
    /// Text error, response bytes could not be converted to string
    TextError(String),
    /// JSON error, received JSON could not be parsed
    JsonError(String),
    /// A response header line could not be parsed
    InvalidHeader(String),
    /// The response status line could not be parsed
    InvalidResponse(String),
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
            RequestError::TlsNotEnabled => {
                write!(f, "TLS feature is not enabled, enable it to use HTTPS")
            }
            RequestError::FileError(err) => write!(f, "File error: {}", err),
            RequestError::TextError(err) => write!(f, "Text decode error: {}", err),
            RequestError::JsonError(err) => write!(f, "JSON error: {}", err),
            RequestError::InvalidHeader(line) => write!(f, "Malformed header line: {}", line),
            RequestError::InvalidResponse(line) => {
                write!(f, "Malformed response status line: {}", line)
            }
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

#[cfg(all(feature = "https", feature = "blocking"))]
impl From<native_tls::HandshakeError<std::net::TcpStream>> for RequestError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(all(feature = "https", feature = "blocking"))]
impl From<native_tls::Error> for RequestError {
    fn from(error: native_tls::Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(all(feature = "https", feature = "async"))]
impl From<tokio_native_tls::native_tls::Error> for RequestError {
    fn from(error: tokio_native_tls::native_tls::Error) -> Self {
        RequestError::ConnectionError(error.to_string())
    }
}

#[cfg(all(feature = "https", feature = "async"))]
impl From<tokio_native_tls::native_tls::HandshakeError<tokio::net::TcpStream>> for RequestError {
    fn from(error: tokio_native_tls::native_tls::HandshakeError<tokio::net::TcpStream>) -> Self {
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

#[cfg(feature = "json")]
impl From<JsonError> for RequestError {
    fn from(error: JsonError) -> Self {
        RequestError::JsonError(error.to_string())
    }
}
