use crate::{
    body::Body,
    client::Client,
    error::{self, RequestError},
    response::{Response, ResponseInfo},
    transport::Transport,
    url::Url,
};
use anyhow::Context;
use bufstream::BufStream;
#[cfg(feature = "https")]
use native_tls::TlsConnector;
use std::{
    io::{self, BufRead, Read, Write},
    net::TcpStream,
    thread::{panicking, sleep},
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
    /// Body of the request [`Option<Body>`]
    pub(crate) body_to_send: Option<Body>,
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
    pub fn new(url: &str, request_type: RequestTypes) -> anyhow::Result<Request> {
        let url = crate::url::Url::build_from_string(url.to_string())
            .with_context(|| "Failed to parse url")?;
        let headers = Vec::new();
        let mut request = Request {
            url: url.clone(),
            request_type,
            content_type: ContentTypes::default(),
            headers,
            body_to_send: None,
            timeout: 5000,
            sent: false,
        };
        request.set_header(
            "Host",
            &format!(
                "{}{}{}",
                url.host,
                if url.port == 443 || url.port == 80 {
                    ""
                } else {
                    ":"
                },
                if url.port == 443 || url.port == 80 {
                    "".to_string()
                } else {
                    url.port.to_string()
                },
            ),
        );
        request.set_header("Connection", "close");
        request.set_header("Cache-Control", "max-age=0");
        request.set_header(
            "User-Agent",
            &format!("Menemen/{}", env!("CARGO_PKG_VERSION")),
        );
        Ok(request)
    }

    /// Builds the request body
    pub(crate) fn build_request_body(&mut self) -> String {
        self.set_header("Content-Type", &self.content_type.clone().get_type());
        //{protocol}://{host}{port}
        format!(
            "{request_type} /{path}{queryParams} HTTP/1.1\r\n\
            {headers}\r\n\r\n",
            request_type = self.request_type.get_type(),
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
    /// let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
    /// request.set_timeout(5000);
    /// ```
    pub fn set_timeout(&mut self, timeout: u64) -> Option<error::RequestError> {
        if self.sent {
            Some(error::RequestError::CantSetHeadersAfterRequestSent)
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
    /// let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
    /// request.set_header("Host", "behemehal.org");
    /// ```
    pub fn set_header(&mut self, key: &str, value: &str) -> Option<error::RequestError> {
        if self.sent {
            Some(error::RequestError::CantSetHeadersAfterRequestSent)
        } else {
            let q = self.headers.iter_mut().find(|h| h.name == key);
            match q {
                Some(header) => {
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

    /// Send the request with body stream
    /*     pub fn send_with_body(&mut self, body: &mut dyn Read) -> Result<Response, error::RequestError> {
           if self.sent {
               return Err(error::RequestError::AlreadySent);
           } else {
               let socket_addr = (self.url.host.clone(), self.url.port);

               match TcpStream::connect(socket_addr) {
                   Ok(mut _tcp_stream) => {
                       _tcp_stream
                           .set_read_timeout(Some(Duration::from_millis(self.timeout)))
                           .unwrap();

                       let mut tcp_stream = if self.url.is_https && cfg!(feature = "https") {
                           #[cfg(feature = "https")]
                           {
                               return Transport::Ssl(BufStream::new(
                                   TlsConnector::new()
                                       .unwrap()
                                       .connect(&self.url.host, _tcp_stream)
                                       .unwrap(),
                               ));
                           }
                           #[cfg(not(feature = "https"))]
                           {
                               return Err("HTTPS feature is not enabled".into());
                           }
                       } else {
                           Transport::Tcp(BufStream::new(_tcp_stream))
                       };
                       let mut cbody = String::new();
                       body.read_to_string(&mut cbody).unwrap();
                       self.set_header("content-length", &cbody.len().to_string());
                       let request_body = self.build_request_body();
                       self.sent = true;
                       tcp_stream.write(request_body.as_bytes()).unwrap();
                       tcp_stream.write(cbody.as_bytes()).unwrap();
                       tcp_stream.write(b"\r\n").unwrap();
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
                                               return Err(error::RequestError::ConnectionError(
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
                                           return Err(error::RequestError::ConnectionError(
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
                   Err(e) => Err(error::RequestError::ConnectionError(e.to_string())),
               }
           }
       }
    */
    /// Set a body to send with the request
    /// ## Parameters
    /// * `body` - The body to send with the request
    pub fn body(&mut self, body: Body) -> &mut Self {
        self.body_to_send = Some(body);
        self
    }

    /// Send the request with blocking
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestError`]
    /*     pub fn send_blocked(&mut self) -> Result<Response, error::RequestError> {
           if self.sent {
               return Err(error::RequestError::AlreadySent);
           } else {
               let socket_addr = (self.url.host.clone(), self.url.port);

               match TcpStream::connect(socket_addr) {
                   Ok(mut _tcp_stream) => {
                       _tcp_stream.set_read_timeout(Some(Duration::from_millis(self.timeout)))?;

                       let mut tcp_stream = if self.url.is_https && cfg!(feature = "https") {
                           #[cfg(feature = "https")]
                           {
                               let connector = TlsConnector::new()
                                   .map_err(|e| RequestError::ConnectionError(e.to_string()))?;
                               let stream = connector.connect(&self.url.host, _tcp_stream)?;

                               Transport::Ssl(BufStream::new(stream))
                           }
                           #[cfg(not(feature = "https"))]
                           {
                               return Err(RequestError::ConnectionError(
                                   "HTTPS feature is not enabled".to_string(),
                               ));
                           }
                      } else {
                           Transport::Tcp(BufStream::new(_tcp_stream))
                       };

                       let request_body = self.build_request_body();
                       self.sent = true;
                       tcp_stream.write(request_body.as_bytes())?;
                       if let Some(body_to_send) = self.body_to_send {
                           io::copy(body_to_send, tcp_stream);
                           //let content_as_str = String::new();
                           self.set_header("Content-Length", &body_to_send.len().to_string());
                       }
                       tcp_stream.write(self.body_to_send.as_ref().unwrap().as_bytes())?;
                       tcp_stream.flush()?;

                       let mut lines = vec![String::new()];
                       loop {
                           let last_line = lines.last_mut().unwrap();
                           let read_byte = tcp_stream.read_line(last_line)?;
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
                                   stream: tcp_stream,
                               });
                           }

                           if read_byte == 0 {
                               return Err(error::RequestError::ConnectionError(
                                   "Connection closed by server".to_string(),
                               ));
                           }

                           lines.push(String::new());
                       }
                   }
                   Err(e) => Err(error::RequestError::ConnectionError(e.to_string())),
               }
           }
       }
    */

    /// Send the request with blocking
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestError`]
/*     pub fn send_blocked(&mut self) -> Result<Response, error::RequestError> {
        let client = Client::new(&self.url.host, self.url.port);
    } */

    /// Send the request with non-blocking async
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestError`]
    pub async fn send(&mut self) -> Result<Transport, error::RequestError> {
        if self.sent {
            return Err(error::RequestError::AlreadySent);
        } else {
            let client = Client::new(self.url.clone());
            let transport = client.send_request(self.url.is_https && cfg!(feature = "https"), self).await?
            Response {
                
            }
        }
    }
}
