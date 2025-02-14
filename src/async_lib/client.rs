use crate::{
    body::BodyType,
    error::RequestError,
    request::{ContentTypes, Header, Request},
    response::{Response, ResponseInfo},
    transport::Transport,
    url::Url,
};
use tokio::io::AsyncWriteExt;
use tokio::{
    io::{AsyncReadExt, BufStream},
    net::TcpStream,
};

#[cfg(feature = "https")]
use native_tls::TlsConnector;

use tokio::io::AsyncBufReadExt;

#[cfg(feature = "https")]
use tokio_native_tls::TlsConnector as TokioTlsConnector;

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
    /// ## Note
    /// This function is only available when the `async` feature is enabled
    pub async fn connect(&self) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", self.url.host, self.url.port);
        let stream = TcpStream::connect(address).await?;
        Ok(stream)
    }

    /// Send a request to the server
    /// ## Parameters
    /// * `request` - The request to send [`Request`]
    /// * `tls` - Whether to use TLS [`bool`]
    /// ## Returns
    /// [`anyhow::Result`] with [`Response`] if the request was successful else [`anyhow::Error`]
    pub async fn send_request(
        &self,
        tls: bool,
        request: &mut Request,
    ) -> Result<Response, RequestError> {
        let stream = self.connect().await?;

        let mut stream = if tls {
            #[cfg(feature = "https")]
            {
                let connector = TlsConnector::new()
                    .map_err(|e| RequestError::ConnectionError(e.to_string()))?;
                let stream = TokioTlsConnector::from(connector)
                    .connect(&self.url.host, stream)
                    .await?;
                Transport::Ssl(BufStream::new(stream))
            }
            #[cfg(not(feature = "https"))]
            {
                return Err(RequestError::TlsNotEnabled);
            }
        } else {
            Transport::Tcp(BufStream::new(stream))
        };

        let mut read_body = None;

        #[cfg(feature = "multipart")]
        if let Some(body_to_send) = &request.body_to_send {
            if let BodyType::MultipartFormData(multipart_form) = &body_to_send.body {
                request.set_header(
                    "Content-Type",
                    &format!("multipart/form-data; boundary={}", multipart_form.boundary),
                );
            }
        }

        if let Some(body_to_send) = &mut request.body_to_send {
            let mut read_vec = Vec::new();
            match &mut body_to_send.body {
                BodyType::Reader(reader) => {
                    reader.read_to_end(&mut read_vec).await?;
                }
                BodyType::Bytes(bytes) => {
                    read_vec = bytes.get_ref().to_vec();
                }
                BodyType::FormData(form_data) => {
                    request.content_type = ContentTypes::FormData;
                    let built_form = form_data.build();
                    read_vec = built_form.into_bytes();
                }
                #[cfg(feature = "multipart")]
                BodyType::MultipartFormData(multipart_form) => {
                    read_vec = multipart_form.build().await?;
                }
            }

            request.set_header("Content-Length", &read_vec.len().to_string());
            read_body = Some(read_vec);
        }

        let built_request = request.build_request_body();
        stream.write_all(built_request.as_bytes()).await?;

        if let Some(read_body) = read_body {
            stream.write_all(&read_body).await?;
        }

        stream.flush().await?;

        let mut lines = vec![String::new()];
        loop {
            let last_line = lines.last_mut().unwrap();
            let read_byte = stream.read_line(last_line).await?;

            if last_line == "\r\n" {
                let response_info = ResponseInfo::parse_response_info(&lines[0].trim_end())?;
                let remaining_lines = lines[1..lines.len() - 1].to_vec();
                let headers = remaining_lines
                    .iter()
                    .map(|x| Header::parse(&x.trim_end()))
                    .collect::<Result<Vec<Header>, anyhow::Error>>()?;
                let request_chunked = headers
                    .iter()
                    .any(|x| x.name == "Transfer-Encoding" && x.value == "chunked");

                return Ok(Response {
                    response_info,
                    headers,
                    stream,
                    consumed: false,
                    request_chunked,
                    current_chunk_size: 0,
                    read_chunk_size: 0,
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
