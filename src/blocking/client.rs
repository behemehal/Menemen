use crate::{
    body::BodyType,
    error::RequestError,
    request::{ContentTypes, Header, Request},
    response::{Response, ResponseInfo},
    transport::Transport,
    url::Url,
};

use std::io::BufReader;
use bytes::BytesMut;
use std::io::{BufRead, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;


#[cfg(feature = "https")]
use native_tls::TlsConnector;

const MAX_REDIRECTS: usize = 10;

fn should_redirect(status_code: u16) -> bool {
    matches!(status_code, 302 | 303 | 307 | 308)
}

fn build_redirect_url(current_url: &Url, location: &str) -> Result<Url, RequestError> {
    if location.contains("://") {
        return Url::build_from_string(location.to_string())
            .map_err(|e| RequestError::ConnectionError(e.to_string()));
    }

    if location.starts_with('/') {
        let scheme = if current_url.is_https { "https" } else { "http" };
        let base = if current_url.port == 80 || current_url.port == 443 {
            format!("{}://{}", scheme, current_url.host)
        } else {
            format!("{}://{}:{}", scheme, current_url.host, current_url.port)
        };
        return Url::build_from_string(format!("{}{}", base, location))
            .map_err(|e| RequestError::ConnectionError(e.to_string()));
    }

    Err(RequestError::ConnectionError(format!(
        "Redirect url is not correct '{}'",
        location
    )))
}

/// Blocking HTTP client bound to a target URL.
#[derive(Debug)]
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
    /// [`Result`] with [`TcpStream`] if the connection was successful else [`RequestError`]
    pub fn connect(&self, timeout_ms: u64) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", self.url.host, self.url.port);
        let timeout = Duration::from_millis(timeout_ms);
        let mut addrs = address.to_socket_addrs()?;
        let first_addr = addrs.next().ok_or(RequestError::CantResolveUrl)?;

        let stream = TcpStream::connect_timeout(&first_addr, timeout).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                RequestError::ConnectionTimeout
            } else {
                RequestError::ConnectionError(e.to_string())
            }
        })?;

        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(stream)
    }

    /// Send a request to the server
    /// ## Parameters
    /// * `request` - The request to send
    /// * `tls` - Whether to use TLS
    /// ## Returns
    /// [`Result`] with [`Response`] if the request was successful else [`RequestError`]
    pub fn send_request(&self, tls: bool, request: &mut Request) -> Result<Response, RequestError> {
        use std::io::Read;

        let mut current_url = self.url.clone();
        let mut redirect_count = 0usize;

        loop {
            let stream = Client::new(current_url.clone()).connect(request.timeout())?;

            let mut stream = if tls {
                #[cfg(feature = "https")]
                {
                    let connector = TlsConnector::new()
                        .map_err(|e| RequestError::ConnectionError(e.to_string()))?;
                    let stream = connector.connect(&current_url.host, stream)?;
                    Transport::Ssl(BufReader::new(stream))
                }
                #[cfg(not(feature = "https"))]
                {
                    return Err(RequestError::TlsNotEnabled);
                }
            } else {
                Transport::Tcp(BufReader::new(stream))
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
                        reader.read_to_end(&mut read_vec)?;
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
                        read_vec = multipart_form.build()?;
                    }
                }

                request.set_header("Content-Length", &read_vec.len().to_string());
                read_body = Some(read_vec);
            }

            let built_request = request.build_request_body();
            stream.write_all(built_request.as_bytes())?;

            if let Some(read_body) = read_body {
                stream.write_all(&read_body)?;
            }

            stream.flush()?;

            let mut lines = vec![String::new()];
            loop {
                let last_line = lines.last_mut().unwrap();
                let read_byte = stream.read_line(last_line)?;

                if last_line == "\r\n" {
                    let response_info = ResponseInfo::parse_response_info(&lines[0].trim_end())?;
                    let remaining_lines = lines[1..lines.len() - 1].to_vec();
                    let headers = remaining_lines
                        .iter()
                        .map(|x| Header::parse(&x.trim_end()))
                        .collect::<Result<Vec<Header>, RequestError>>()?;

                    let redirected_location = headers
                        .iter()
                        .find(|x| x.name.eq_ignore_ascii_case("Location"));

                    if request.follow_redirects()
                        && should_redirect(response_info.status_code)
                        && redirected_location.is_some()
                    {
                        if redirect_count >= MAX_REDIRECTS {
                            return Err(RequestError::ConnectionError(
                                "Too many redirects".to_string(),
                            ));
                        }

                        let new_url = build_redirect_url(
                            &current_url,
                            redirected_location.unwrap().value.as_str(),
                        )?;
                        current_url = new_url.clone();
                        request.set_url(new_url);
                        redirect_count += 1;
                        break;
                    }

                    let request_chunked = headers.iter().any(|x| {
                        x.name.eq_ignore_ascii_case("Transfer-Encoding")
                            && x.value
                                .split(',')
                                .any(|v| v.trim().eq_ignore_ascii_case("chunked"))
                    });

                    return Ok(Response {
                        response_info,
                        headers,
                        stream,
                        consumed: false,
                        request_chunked,
                        current_chunk_size: 0,
                        read_chunk_size: 0,
                        chunk_parse_buffer: BytesMut::new(),
                        crlf_skip_done: false,
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
}
