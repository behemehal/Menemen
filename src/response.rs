use core::panic;

use crate::transport::Transport;
use anyhow::Context;

#[derive(Clone, Debug, Default)]
pub struct ResponseInfo {
    pub http_version: String,
    pub status_code: u16,
    pub status_message: String,
}

impl ResponseInfo {
    //Parse coming HTTP/1.1 200 OK answer to ResponseInfo struct
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

#[derive(Debug)]
pub struct ResponseReader {
    pub stream: Transport,
    pub read_len: u64,
    pub content_len: i64,
}

impl ResponseReader {
    pub fn new(stream: Transport, content_len: i64) -> ResponseReader {
        ResponseReader {
            stream,
            read_len: 0,
            content_len,
        }
    }

    pub fn complete(&self) -> bool {
        if self.content_len < 0 {
            false
        } else {
            self.read_len == (self.content_len as u64)
        }
    }
}

impl std::io::Read for ResponseReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.stream.read(buf);
        match read {
            Ok(_) => {
                self.read_len += 1;
                read
            }
            Err(_) => {
                read
            }
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub response_info: ResponseInfo,
    pub headers: Vec<crate::request::Header>,
    pub stream: ResponseReader,
}
