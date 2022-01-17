use crate::{error, response::Response, response::ResponseInfo, transport::Transport, url::Url};
use anyhow::Context;
use bufstream::BufStream;
use openssl::ssl::{SslConnector, SslMethod};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

/// HTTP Header
/// ##### [https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers]
#[derive(Debug, Clone)]
pub struct Header {
    /// The name of the header
    pub name: String,
    /// The value of the header
    pub value: String,
}

impl Header {
    /// Parse raw http header response to [`Header`] struct
    /// ## Parameters
    /// * `line` - The raw http header response
    /// ## Returns
    /// [`Header`] if the header was successfully parsed else [`error::Error`]
    /// ## Example
    /// ```
    /// use menemen::request::Header;
    /// let header = Header::parse("Content-Type: text/html; charset=utf-8").unwrap();
    /// assert_eq!(header.name.clone(), "Content-Type");
    /// assert_eq!(header.value, "text/html; charset=utf-8");
    /// ```
    pub fn parse(line: &str) -> anyhow::Result<Header> {
        if !line.contains(":") {
            return Err(anyhow::anyhow!("Failed to parse response info"));
        }
        let parts = line.split(": ").collect::<Vec<_>>();
        let name = parts[0].to_string();
        let value = if parts.len() == 1 {
            String::new()
        } else {
            parts[1].to_string()
        };
        Ok(Header { name, value })
    }
}

/// List of RequestTypes
/// #### https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
#[derive(Debug)]
pub enum RequestTypes {
    /// GET Method
    GET,
    /// POST Method
    POST,
    /// PUT Method
    PUT,
    /// DELETE Method
    DELETE,
}

impl RequestTypes {
    /// Get the string representation of the RequestType
    pub fn get_type(&self) -> String {
        match self {
            RequestTypes::GET => "GET".to_string(),
            RequestTypes::POST => "POST".to_string(),
            RequestTypes::PUT => "PUT".to_string(),
            RequestTypes::DELETE => "DELETE".to_string(),
        }
    }
}

/// ContentTypes
/// #### https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
#[derive(Clone, Debug)]
pub enum ContentTypes {
    /// application/json
    JSON,
    /// text/html
    HTML,
    /// text/plain
    Text,
    /// image/png
    Png,
    /// audio/mp3
    MP3,
    /// text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
    Any,
    /// application/octet-stream
    OctetStream,
}

impl Default for ContentTypes {
    fn default() -> Self {
        ContentTypes::Any
    }
}

impl ContentTypes {
    /// Get the string representation of the ContentType
    /// ## Example
    /// ```
    /// use menemen::request::ContentTypes;
    /// let content_type = ContentTypes::JSON;
    /// assert_eq!(content_type.get_type(), "application/json");
    /// ```
    pub fn get_type(&self) -> &str {
        match self {
            ContentTypes::JSON => "application/json",
            ContentTypes::HTML => "text/html",
            ContentTypes::Text => "text/plain",
            ContentTypes::Png => "image/png",
            ContentTypes::MP3 => "audio/mp3",
            ContentTypes::Any => "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ContentTypes::OctetStream => "application/octet-stream",
        }
    }
}

/// Request struct
#[derive(Debug)]
pub struct Request {
    /// Url of the request [`Url`]
    url: Url,
    request_type: RequestTypes,
    /// ContentType of the request [`ContentTypes`]
    pub content_type: ContentTypes,
    /// Headers of the request [`Vec<Header>`]
    headers: Vec<Header>,
    /// Timeout of the request [`u64`]
    timeout: u64,
    /// Is the request sent
    sent: bool,
}

impl Request {
    /// Create a new [`Request`]
    /// ## Parameters
    /// * `url` - The url to send the request to
    /// * `request_type` - The type of request to send takes [`RequestTypes`]
    /// ## Returns
    /// [`Request`] if the request was successfully created else [`error::Error`]
    pub fn new(url: &'static str, request_type: RequestTypes) -> anyhow::Result<Request> {
        let url = crate::url::Url::build_from_string(url.to_string())
            .with_context(|| "Failed to parse url")?;
        let headers = Vec::new();
        let mut request = Request {
            url: url.clone(),
            request_type,
            content_type: ContentTypes::default(),
            headers,
            timeout: 5000,
            sent: false,
        };
        request.set_header("Host", &format!("{}:{}", url.host, url.port));
        request.set_header("Connection", "close");
        request.set_header("Cache-Control", "max-age=0");
        request.set_header(
            "User-Agent",
            &format!("Menemen/{}", env!("CARGO_PKG_VERSION")),
        );
        Ok(request)
    }

    /// Builds the request body
    pub fn build_request_body(&mut self) -> String {
        self.set_header("Content-Type", &self.content_type.clone().get_type());
        format!(
            "{request_type} {protocol}://{host}:{port}/{path}{queryParams} HTTP/1.1\r\n\
            {headers}\r\n\r\n",
            request_type = self.request_type.get_type(),
            protocol = if self.url.is_https { "https" } else { "http" },
            host = self.url.host,
            port = self.url.port,
            path = self.url.paths.join("/"),
            queryParams = if self.url.query_params.is_empty() {
                "".to_owned()
            } else {
                "?".to_owned() + &self.url.join_query_params()
            },
            headers = self
                .headers
                .iter()
                .map(|x| format!("{}:{}", x.name, x.value))
                .collect::<Vec<_>>()
                .join("\r\n")
        )
    }

    /// Set timeout for the request
    /// ## Parameters
    /// * `timeout` - The timeout in milliseconds
    /// ## Returns
    /// [`Request`] if the timeout set before the request sent else [`error::Error`]
    /// ## Example
    /// ```
    /// use menemen::request::{Request, RequestTypes};
    ///
    /// let mut request = Request::new("https://behemehal.net/test", RequestTypes::GET).unwrap();
    /// request.set_timeout(5000);
    /// ```
    pub fn set_timeout(&mut self, timeout: u64) -> Option<error::RequestErrors> {
        if self.sent {
            Some(error::RequestErrors::CantSetHeadersAfterRequestSent)
        } else {
            self.timeout = timeout;
            None
        }
    }

    /// Get headers of the request
    /// ## Returns
    /// [`Vec<Header>`]
    pub fn get_headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    /// Get header for the request
    /// ## Parameters
    /// * `key` - The name of the header
    /// ## Returns
    /// [`String`] if the header exists else [`None`]
    pub fn get_header(&self, key: &str) -> Option<Header> {
        self.headers.clone().into_iter().find(|h| h.name == key)
    }

    /// Set header for the request
    /// ## Parameters
    /// * `key` - The name of the header
    /// * `value` - The value of the header
    /// ## Returns
    /// [`Request`] if the header was set before the request sent else [`error::Error`]
    /// ## Example
    /// ```
    /// use menemen::request::{Request, RequestTypes};
    ///
    /// let mut request = Request::new("https://behemehal.net/test", RequestTypes::GET).unwrap();
    /// request.set_header("Host", "behemehal.net");
    /// ```
    pub fn set_header(&mut self, key: &str, value: &str) -> Option<error::RequestErrors> {
        if self.sent {
            Some(error::RequestErrors::CantSetHeadersAfterRequestSent)
        } else {
            match self.headers.iter_mut().find(|h| h.name == key) {
                Some(ref mut header) => {
                    header.value = value.to_string();
                }
                None => {
                    self.headers.push(Header {
                        name: key.to_string(),
                        value: value.to_string(),
                    });
                }
            }
            None
        }
    }

    /// Send the request with body stream [NotImplemented]
    pub fn send_with_body(&mut self) {
        unimplemented!("Not supported yet")
    }

    /// Send the request without body stream
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestErrors`]
    pub fn send(&mut self) -> Result<Response, error::RequestErrors> {
        if self.sent {
            return Err(error::RequestErrors::AlreadySent);
        } else {
            let socket_addr = (self.url.host.clone(), self.url.port);
            let connector = if self.url.is_https {
                Some(match SslConnector::builder(SslMethod::tls()) {
                    Ok(e) => e.build(),
                    Err(_) => {
                        return Err(error::RequestErrors::ConnectionError(
                            "Failed to establish ssl connection".to_string(),
                        ));
                    }
                })
            } else {
                None
            };
            match TcpStream::connect(socket_addr) {
                Ok(mut _tcp_stream) => {
                    let mut tcp_stream = if self.url.is_https {
                        _tcp_stream
                            .set_read_timeout(Some(Duration::from_millis(5000)))
                            .unwrap();
                        Transport::Ssl(BufStream::new(
                            connector
                                .unwrap()
                                .connect(&self.url.host, _tcp_stream)
                                .unwrap(),
                        ))
                    } else {
                        //_tcp_stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
                        Transport::Tcp(BufStream::new(_tcp_stream))
                    };
                    self.sent = true;
                    let request_body = self.build_request_body();
                    tcp_stream.write(request_body.as_bytes()).unwrap();
                    tcp_stream.flush().unwrap();

                    let mut lines = vec![String::new()];
                    let mut new_line = false;
                    let mut connection_info_collected = false;
                    let mut connection_info = ResponseInfo::default();
                    let mut headers: Vec<Header> = Vec::new();
                    let mut last_char = '\0';
                    loop {
                        let mut buffer = [0; 1];
                        tcp_stream.read(&mut buffer).unwrap();
                        //Convert byte to char
                        let cchar = char::from(buffer[0]);
                        //If its a line break
                        if last_char == '\r' && cchar == '\n' {
                            //If newline used again collect body
                            if new_line {
                                for line in &lines {
                                    match Header::parse(line) {
                                        Ok(header_line) => {
                                            headers.push(header_line);
                                        }
                                        Err(_) => {
                                            return Err(error::RequestErrors::ConnectionError(
                                                "Malformed response header".to_string(),
                                            ));
                                        }
                                    }
                                }
                                return Ok(Response {
                                    response_info: connection_info,
                                    headers,
                                    stream: tcp_stream,
                                });
                            } else {
                                if !connection_info_collected {
                                    if let Ok(con_info) =
                                        ResponseInfo::parse_response_info(&lines[0])
                                    {
                                        connection_info = con_info;
                                        connection_info_collected = true;
                                        lines = Vec::new();
                                    } else {
                                        return Err(error::RequestErrors::ConnectionError(
                                            "Malformed response".to_string(),
                                        ));
                                    }
                                }
                                new_line = true;
                            }
                        } else {
                            //If coming line is \r dont reset 'new_line'
                            if cchar != '\r' {
                                if new_line {
                                    lines.push(String::new());
                                }
                                let line_len = lines.len();
                                lines[line_len - 1] += &cchar.to_string();
                                new_line = false;
                            }
                        }
                        last_char = cchar;
                    }
                }
                Err(e) => Err(error::RequestErrors::ConnectionError(e.to_string())),
            }
        }
    }
}
