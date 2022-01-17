use crate::transport::Transport;
use anyhow::Context;
use crate::request;

/// ResponseInfo struct
#[derive(Clone, Debug, Default)]
pub struct ResponseInfo {
    /// The HTTP version of the server
    pub http_version: String,
    /// The status code of the response
    pub status_code: u16,
    /// The status message of the response
    pub status_message: String,
}

impl ResponseInfo {
    /// Parse coming HTTP/1.1 200 OK answer to ResponseInfo struct
    /// ## Parameters
    /// * `response` - The HTTP/1.1 200 OK answer
    /// ## Returns
    /// [`ResponseInfo`] if the answer was successfully parsed else [`anyhow::Error`]
    pub fn parse_response_info(response: &str) -> Result<ResponseInfo, anyhow::Error> {
        let mut response_info = ResponseInfo {
            http_version: String::new(),
            status_code: 0,
            status_message: String::new(),
        };
        let response_info_vec: Vec<&str> = response.split(" ").collect();
        if response_info_vec.len() < 2 {
            return Err(anyhow::anyhow!("Failed to parse response info"));
        }
        response_info.http_version = response_info_vec[0].to_string();
        response_info.status_code = response_info_vec[1].parse::<u16>().with_context(|| {
            format!(
                "Failed to parse status code from response: {}",
                response_info_vec[1]
            )
        })?;
        response_info.status_message = response_info_vec[2..].join(" ");
        Ok(response_info)
    }
}

/// [`Response`] struct contains incoming headers ([`Vec<request::Header>`]), [`ResponseInfo`], and stream ([`Transport`]) which implements [`std::io::Read`], [`std::io::Write`] and [`std::io::BufRead`]
#[allow(missing_debug_implementations)]
pub struct Response {
    /// Response info [`ResponseInfo`]
    pub response_info: ResponseInfo,
    /// Response headers [`Vec<request::Header>`]
    pub headers: Vec<request::Header>,
    /// Incoming body stream
    pub stream: Transport,
}
