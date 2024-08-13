use crate::{
    error::RequestError, request::Request, response::Response, transport::Transport, url::Url,
};

#[cfg(not(feature = "async"))]
use std::io::{Read, Write, copy, BufRead};

#[cfg(feature = "async")]
use tokio::{io::{BufStream}, net::TcpStream};

#[cfg(not(feature = "async"))]
use std::net::TcpStream;

#[cfg(not(feature = "async"))]
use bufstream::BufStream;

#[cfg(feature = "https")]
use native_tls::TlsConnector;
#[cfg(feature = "async")]
use tokio::io::AsyncBufReadExt;
#[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use crate::request::Header;
use crate::response::ResponseInfo;

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
        Self { url }
    }

    /// Connect to the server
    /// ## Returns
    /// [`anyhow::Result`] with [`TcpStream`] if the connection was successful else [`anyhow::Error`]
    #[cfg(not(feature = "async"))]
    pub fn connect(&self) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", self.url.host, self.url.port);
        let stream = TcpStream::connect(address)?;
        Ok(stream)
    }

    /// Connect to the server
    /// ## Returns
    /// [`anyhow::Result`] with [`TcpStream`] if the connection was successful else [`anyhow::Error`]
    /// ## Note
    /// This function is only available when the `async` feature is enabled
    #[cfg(feature = "async")]
    pub async fn connect(&self) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", self.url.host, self.url.port);
        let stream = TcpStream::connect(address).await?;
        Ok(stream)
    }

    /// Send a request to the server
    /// ## Parameters
    /// * `request` - The request to send
    /// * `tls` - Whether to use TLS
    /// ## Returns
    /// [`anyhow::Result`] with [`Response`] if the request was successful else [`anyhow::Error`]
    #[cfg(not(feature = "async"))]
    pub fn send_request(&self, tls: bool, request: &mut Request) -> Result<Response, RequestError> {
        let stream = self.connect()?;

        let mut stream = if tls && cfg!(feature = "https") {
            #[cfg(feature = "https")] {
                let connector =
                    TlsConnector::new().map_err(|e| RequestError::ConnectionError(e.to_string()))?;
                let stream = connector.connect(&self.url.host, stream)?;
                Transport::Ssl(BufStream::new(stream))
            }
            unreachable!()
        } else {
            Transport::Tcp(BufStream::new(stream))
        };

        if let Some(ref mut body_to_send) = &mut request.body_to_send {
            if let Some(size_hint) = body_to_send.size_hint() {
                println!("Setting Content-Length: {}", size_hint);
                request.set_header("Content-Length", &size_hint.to_string());
            }
        }

        stream
            .write_all(request.build_request_body().as_bytes())?;

        if let Some(ref mut body_to_send) = &mut request.body_to_send {
            copy(body_to_send, &mut stream)?;
        }
        stream.flush()?;

        let mut lines = vec![String::new()];
        loop {
            let last_line = lines.last_mut().unwrap();
            let read_byte = stream.read_line(last_line)?;

            if last_line == "\r\n" {
                let response_info =
                    ResponseInfo::parse_response_info(&lines[0].replace("\r\n", ""))?;
                let remaining_lines = lines[1..lines.len() - 2].to_vec();
                let headers = remaining_lines
                    .iter()
                    .map(|x| Header::parse(&x.replace("\r\n", "")))
                    .collect::<Result<Vec<Header>, anyhow::Error>>()?;
                return Ok(Response {
                    response_info,
                    headers,
                    stream,
                    consumed: true,
                });
            }

            if read_byte == 0 {
                return Err(RequestError::ConnectionError(
                    "Connection closed by server".to_string(),
                ));
            }

            lines.push(String::new());
        }
    }

    /// Send a request to the server
    /// ## Parameters
    /// * `request` - The request to send [`Request`]
    /// * `tls` - Whether to use TLS [`bool`]
    /// ## Returns
    /// [`anyhow::Result`] with [`Response`] if the request was successful else [`anyhow::Error`]
    #[cfg(feature = "async")]
    pub async fn send_request(
        &self,
        tls: bool,
        request: &mut Request,
    ) -> Result<Response, RequestError> {
        use tokio::io::AsyncWriteExt;

        let stream = self.connect().await?;

        let mut stream = if tls && cfg!(all(feature = "https", feature = "https-async")) {
            #[cfg(all(feature = "https", feature = "https-async"))]
            {
                let connector = TlsConnector::new()
                    .map_err(|e| RequestError::ConnectionError(e.to_string()))?;
                let stream = TokioTlsConnector::from(connector)
                    .connect(&self.url.host, stream)
                    .await?;
                Transport::Ssl(BufStream::new(stream));
            }
            unreachable!()
        } else {
            Transport::Tcp(BufStream::new(stream))
        };

        if let Some(ref mut body_to_send) = &mut request.body_to_send {
            if let Some(size_hint) = body_to_send.size_hint() {
                println!("Setting Content-Length: {}", size_hint);
                request.set_header("Content-Length", &size_hint.to_string());
            }
        }

        stream
            .write_all(request.build_request_body().as_bytes())
            .await?;

        if let Some(ref mut body_to_send) = &mut request.body_to_send {
            tokio::io::copy(body_to_send, &mut stream).await?;
        }
        stream.flush().await?;

        let mut lines = vec![String::new()];
        loop {
            let last_line = lines.last_mut().unwrap();
            let read_byte = stream.read_line(last_line).await?;

            if last_line == "\r\n" {
                let response_info =
                    ResponseInfo::parse_response_info(&lines[0].replace("\r\n", ""))?;
                let remaining_lines = lines[1..lines.len() - 2].to_vec();
                let headers = remaining_lines
                    .iter()
                    .map(|x| Header::parse(&x.replace("\r\n", "")))
                    .collect::<Result<Vec<Header>, anyhow::Error>>()?;
                return Ok(Response {
                    response_info,
                    headers,
                    stream,
                    consumed: true,
                });
            }

            if read_byte == 0 {
                return Err(RequestError::ConnectionError(
                    "Connection closed by server".to_string(),
                ));
            }

            lines.push(String::new());
        }
    }
}
