use crate::{error::RequestError, request::Request, response::Response, transport::Transport, url::Url};
use std::{io::{Read, Write}, net::TcpStream};

use bufstream::BufStream;
#[cfg(feature = "https")]
use native_tls::TlsConnector;

pub struct Client {
    url: Url,
}

impl Client {
    /// Create a new client
    /// ## Parameters
    /// * `address` - The client's address
    /// * `port` - The client's port
    /// ## Returns
    /// [`Client`]
    pub fn new(url: Url) -> Self {
        Self {
            url,
        }
    }

    /// Connect to the server
    /// ## Returns
    /// [`anyhow::Result`] with [`TcpStream`] if the connection was successful else [`anyhow::Error`]
    pub fn connect(&self) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", self.url.host, self.url.port);
        let stream = TcpStream::connect(address)?;
        Ok(stream)
    }

    /// Send a request to the server
    /// ## Parameters
    /// * `request` - The request to send
    /// * `tls` - Whether to use TLS
    /// ## Returns
    /// [`anyhow::Result`] with [`Response`] if the request was successful else [`anyhow::Error`]
    pub fn send_request(&self, tls: bool, request: &Request) -> Result<Response, RequestError> {
        let mut stream = self.connect()?;

        let stream = if tls {
            let connector =
                TlsConnector::new().map_err(|e| RequestError::ConnectionError(e.to_string()))?;
            let stream = connector.connect(&self.url.host, stream)?;
            Transport::Ssl(BufStream::new(stream))
        } else {
            Transport::Tcp(BufStream::new(stream))
        };


        stream.write_all(request.build_request_body().as_bytes())?;
        if let Some(body_to_send) = request.body_to_send {
            stream.write_all(body_to_send.bytes())?;
        }
        stream.flush()?;

        Ok(response)
    }
}
