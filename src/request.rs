use crate::{body::Body, client::Client, error::RequestError, response::Response, url::Url};
use anyhow::Context;

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
    /// Form-Data
    FormData,
    /// Multipart-Form-Data
    MultipartFormData,
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
            ContentTypes::FormData => "application/x-www-form-urlencoded",
            ContentTypes::MultipartFormData => "multipart/form-data",
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
    pub fn set_timeout(&mut self, timeout: u64) -> Option<RequestError> {
        if self.sent {
            Some(RequestError::CantSetHeadersAfterRequestSent)
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
    pub fn set_header(&mut self, key: &str, value: &str) -> Option<RequestError> {
        if self.sent {
            Some(RequestError::CantSetHeadersAfterRequestSent)
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

    /// Append body to the request
    /// ## Parameters
    /// * `body` - The body to send with the request
    pub fn append_body(&mut self, body: Body) -> &mut Self {
        self.body_to_send = Some(body);
        self
    }

    /// Send the request with non-blocking async
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestError`]
    #[cfg(feature = "async")]
    pub async fn send(&mut self) -> Result<Response, RequestError> {
        if self.sent {
            return Err(RequestError::AlreadySent);
        } else {
            let client = Client::new(self.url.clone());
            client
                .send_request(self.url.is_https && cfg!(feature = "https"), self)
                .await
        }
    }

    /// Send the request with non-blocking async
    /// ## Returns
    /// [`Response`] if the request was sent successfully else [`error::RequestError`]
    #[cfg(not(feature = "async"))]
    pub fn send(&mut self) -> Result<Response, RequestError> {
        if self.sent {
            return Err(RequestError::AlreadySent);
        } else {
            let client = Client::new(self.url.clone());
            client.send_request(self.url.is_https && cfg!(feature = "https"), self)
        }
    }
}
