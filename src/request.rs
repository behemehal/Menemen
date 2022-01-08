use anyhow::Context;
use dns_lookup::lookup_host;
use openssl::ssl::{SslConnector, SslMethod};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{error, response::Response};
use crate::{
    response::{ResponseInfo, ResponseReader},
    transport::Transport,
};

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    //Parse coming header to Header struct
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

pub enum RequestTypes {
    GET,
    POST,
    PUT,
    DELETE,
}

impl RequestTypes {
    pub fn get_type(&self) -> String {
        match self {
            RequestTypes::GET => "GET".to_string(),
            RequestTypes::POST => "POST".to_string(),
            RequestTypes::PUT => "PUT".to_string(),
            RequestTypes::DELETE => "DELETE".to_string(),
        }
    }
}

#[derive(Clone)]
pub enum ContentTypes {
    JSON,
    HTML,
    Text,
    Png,
    MP3,
    Any,
    OctetStream,
}

impl Default for ContentTypes {
    fn default() -> Self {
        ContentTypes::Any
    }
}

impl ContentTypes {
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

pub struct Request {
    url: crate::url::Url,
    request_type: RequestTypes,
    pub content_type: ContentTypes,
    headers: Vec<Header>,
    timeout: u64,
    sent: bool,
}

impl Request {
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

    pub fn set_timeout(&mut self, timeout: u64) -> Option<error::RequestErrors> {
        if self.sent {
            Some(error::RequestErrors::CantSetHeadersAfterRequestSent)
        } else {
            self.timeout = timeout;
            None
        }
    }

    pub fn get_headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn get_header(&self, key: &str) -> Option<Header> {
        self.headers.clone().into_iter().find(|h| h.name == key)
    }

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

    pub fn send_with_body(&mut self) {
        unimplemented!("Not supported yet")
    }

    pub fn send(&mut self) -> Result<Response, error::RequestErrors> {
        if self.sent {
            return Err(error::RequestErrors::AlreadySent);
        } else {
            match lookup_host(&self.url.host) {
                Ok(ip_addr) => {
                    let socket_addr = SocketAddr::new(ip_addr[0], self.url.port);

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
                                Transport::Ssl(
                                    connector
                                        .unwrap()
                                        .connect(&self.url.host, _tcp_stream)
                                        .unwrap(),
                                )
                            } else {
                                _tcp_stream
                                    .set_read_timeout(Some(Duration::from_millis(5000)))
                                    .unwrap();
                                Transport::Tcp(_tcp_stream)
                            };

                            self.sent = true;
                            let request_body = self.build_request_body();
                            tcp_stream.write(request_body.as_bytes()).unwrap();
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
                                                    return Err(
                                                        error::RequestErrors::ConnectionError(
                                                            "Malformed response header".to_string(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                        let content_len = match headers
                                            .iter_mut()
                                            .find(|h| h.name == "Content-Length")
                                        {
                                            Some(header) => match header.value.parse::<i64>() {
                                                Ok(d) => d,
                                                Err(_) => -1,
                                            },
                                            None => -1,
                                        };
                                        return Ok(Response {
                                            response_info: connection_info,
                                            headers,
                                            stream: ResponseReader::new(tcp_stream, content_len),
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
                Err(_) => Err(error::RequestErrors::CantResolveUrl),
            }
        }
    }
}
