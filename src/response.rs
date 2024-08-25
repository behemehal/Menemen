use std::{fmt::Debug, thread::sleep, time::Duration};

use crate::error::RequestError;
use crate::request;
use crate::transport::Transport;
use anyhow::Context;

#[cfg(feature = "async")]
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

#[cfg(not(feature = "async"))]
use std::io::{BufRead, Read};

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
    /// Flag that indicates if the response is already consumed
    pub(crate) consumed: bool,
    pub(crate) request_chunked: bool,
    pub(crate) current_chunk_size: usize,
    pub(crate) read_chunk_size: usize,
}

impl Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("response_info", &self.response_info)
            .field("headers", &self.headers)
            .finish()
    }
}

impl Response {
    //TODO: Implement AsyncRead instead
    #[cfg(feature = "async")]
    pub async fn read_to_end(&mut self) -> Result<Vec<u8>, RequestError> {
        self.consumed = true;
        if self.request_chunked {
            let mut chunk_size = self.read_chunk_size(false).await?;
            let mut read_data = Vec::new();

            while chunk_size > 0 {
                let mut read_chunk = vec![0; chunk_size];
                let read_byte = self.stream.read_exact(&mut read_chunk).await?;

                if read_byte == 0 {
                    return Err(RequestError::ConnectionError(
                        "Connection closed by server".to_string(),
                    ));
                }

                chunk_size = self.read_chunk_size(true).await?;
                read_data.extend(read_chunk);
            }

            Ok(read_data)
        } else {
            let mut read_data = Vec::new();
            self.stream.read_to_end(&mut read_data).await?;
            Ok(read_data)
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn _read_to_end(&mut self) -> Result<Vec<u8>, RequestError> {
        self.consumed = true;
        if self.request_chunked {
            let mut chunk_size = self.read_chunk_size(false)?;
            let mut read_data = Vec::new();

            while chunk_size > 0 {
                let mut read_chunk = vec![0; chunk_size];
                self.stream.read_exact(&mut read_chunk)?;

                chunk_size = self.read_chunk_size(true)?;
                read_data.extend(read_chunk);
            }

            Ok(read_data)
        } else {
            let mut read_data = Vec::new();
            self.stream.read_to_end(&mut read_data)?;
            Ok(read_data)
        }
    }

    #[cfg(feature = "async")]
    pub async fn text(&mut self) -> Result<String, RequestError> {
        let content_encoding = self
            .headers
            .iter()
            .find(|x| x.name == "Content-Encoding")
            .map(|x| x.value.as_str());

        if let Some("gzip") = content_encoding {
            #[cfg(feature = "gzip")]
            {
                use libflate::gzip::Decoder;
                use std::io::{Cursor, Read};

                let read_data = self.read_to_end().await?;
                let read_data = Cursor::new(read_data);
                let mut decoder =
                    Decoder::new(read_data).map_err(|e| RequestError::TextError(e.to_string()))?;

                let decoded_data = tokio::task::spawn_blocking(move || {
                    let mut decoded_data = Vec::new();

                    if let Err(error) = decoder.read_to_end(&mut decoded_data) {
                        return Err(error);
                    }

                    Ok(decoded_data)
                })
                .await
                .map_err(|e| RequestError::TextError(e.to_string()))??;

                String::from_utf8(decoded_data).map_err(|e| RequestError::TextError(e.to_string()))
            }

            #[cfg(not(feature = "gzip"))]
            {
                Err(RequestError::TextError(
                    "Gzip support is disabled".to_string(),
                ))
            }
        } else {
            let read_data = self.read_to_end().await?;
            String::from_utf8(read_data).map_err(|e| RequestError::TextError(e.to_string()))
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn text(&mut self) -> Result<String, RequestError> {
        let content_encoding = self
            .headers
            .iter()
            .find(|x| x.name == "Content-Encoding")
            .map(|x| x.value.as_str());

        if let Some("gzip") = content_encoding {
            #[cfg(feature = "gzip")]
            {
                use libflate::gzip::Decoder;
                use std::io::Cursor;

                let read_data = self._read_to_end()?;
                let read_data = Cursor::new(read_data);
                let mut decoder =
                    Decoder::new(read_data).map_err(|e| RequestError::TextError(e.to_string()))?;

                let mut decoded_data = Vec::new();

                decoder.read_to_end(&mut decoded_data)?;
                String::from_utf8(decoded_data).map_err(|e| RequestError::TextError(e.to_string()))
            }

            #[cfg(not(feature = "gzip"))]
            {
                Err(RequestError::TextError(
                    "Gzip support is disabled".to_string(),
                ))
            }
        } else {
            let mut str_buffer = Vec::new();
            println!("Reading response text");
            self.read_to_end(&mut str_buffer)?;

            Ok(
                String::from_utf8(str_buffer)
                    .map_err(|e| RequestError::TextError(e.to_string()))?,
            )
        }
    }

    #[cfg(all(feature = "async", feature = "json"))]
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, RequestError> {
        self.consumed = true;
        let string = self.text().await?;
        let json: T = serde_json::from_str(&string)?;
        Ok(json)
    }

    #[cfg(all(not(feature = "async"), feature = "json"))]
    pub fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, RequestError> {
        self.consumed = true;
        let string = self.text()?;
        let json: T = serde_json::from_str(&string)?;
        Ok(json)
    }

    #[cfg(feature = "async")]
    async fn read_chunk_size(&mut self, double_read: bool) -> Result<usize, std::io::Error> {
        let mut chunk_size_string = String::new();

        // If double_read is true, read the first line to get the chunk size
        if double_read {
            let read_byte = self.stream.read_line(&mut String::new()).await?;
            if read_byte == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Connection closed by server",
                ));
            }
        }

        let read_byte = self.stream.read_line(&mut chunk_size_string).await?;
        if read_byte == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "Connection closed by server",
            ));
        }

        Ok(usize::from_str_radix(&chunk_size_string.trim_end(), 16).unwrap())
    }

    #[cfg(not(feature = "async"))]
    fn read_chunk_size(&mut self, double_read: bool) -> Result<usize, std::io::Error> {
        let mut chunk_size_string = String::new();

        // If double_read is true, read the first line to get the chunk size
        if double_read {
            let read_byte = self.stream.read_line(&mut String::new())?;
            if read_byte == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Connection closed by server",
                ));
            }
        }

        let read_byte = self.stream.read_line(&mut chunk_size_string)?;
        if read_byte == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "Connection closed by server",
            ));
        }

        Ok(usize::from_str_radix(&chunk_size_string.trim_end(), 16).unwrap())
    }
}

#[cfg(not(feature = "async"))]
impl Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        println!("Trying to fill buffer: {:?}", buf.len());
        if self.request_chunked {
            if self.current_chunk_size == 0 && self.read_chunk_size == 0 {
                self.current_chunk_size = self.read_chunk_size(false)?;
                println!("Reading first ever chunk size: {}", self.current_chunk_size);
            }

            let mut read_buffer_len = 0;
            println!("Remaining buffer len: {}", read_buffer_len);
            while read_buffer_len != buf.len() {
                println!(
                    "While: current_chunk_size: {}, read_chunk_size: {}",
                    self.current_chunk_size, self.read_chunk_size
                );
                if self.current_chunk_size == self.read_chunk_size {
                    self.current_chunk_size = self.read_chunk_size(true)?;
                    self.read_chunk_size = 0;

                    println!("READ NEXT: Read chunk size: {}", self.current_chunk_size);
                    if self.current_chunk_size == 0 {
                        println!("End of chunked data");
                        return Ok(read_buffer_len);
                    }
                }

                let remaining_chunk_size = self.current_chunk_size - self.read_chunk_size;

                let pointer = remaining_chunk_size.min(buf.len());

                let read_byte = self.stream.read(&mut buf[..pointer]).unwrap();
                self.read_chunk_size += read_byte;
                read_buffer_len += read_byte;
                println!("READ: Read byte: {}", read_byte);
                sleep(Duration::from_millis(100));
            }
            Ok(buf.len())
        } else {
            self.stream.read(buf)
        }
    }
}
