use crate::error::RequestError;
use crate::request;
use crate::transport::Transport;

use bytes::BytesMut;
use pin_project_lite::pin_project;
#[cfg(feature = "async")]
use tokio::io::AsyncReadExt;
// read_line is only needed by the gzip-only chunk-size helper.
#[cfg(all(feature = "async", feature = "gzip"))]
use tokio::io::AsyncBufReadExt;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, ReadBuf};

use std::fmt::Debug;
#[cfg(not(feature = "async"))]
use std::io::{BufRead, Read};

#[cfg(feature = "async")]
use std::task::Poll;
#[cfg(feature = "async")]
use std::{io, pin::Pin, task::Context};

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
    /// [`ResponseInfo`] if the answer was successfully parsed else [`RequestError::InvalidResponse`]
    pub fn parse_response_info(response: &str) -> Result<ResponseInfo, RequestError> {
        let mut response_info = ResponseInfo {
            http_version: String::new(),
            status_code: 0,
            status_message: String::new(),
        };
        let response_info_vec: Vec<&str> = response.split(" ").collect();
        if response_info_vec.len() < 2 {
            return Err(RequestError::InvalidResponse(response.to_string()));
        }
        response_info.http_version = response_info_vec[0].to_string();
        response_info.status_code = response_info_vec[1]
            .parse::<u16>()
            .map_err(|_| RequestError::InvalidResponse(response.to_string()))?;
        response_info.status_message = response_info_vec[2..].join(" ");
        Ok(response_info)
    }
}

pin_project! {
    /// HTTP response with headers, status metadata, and a readable body stream.
    #[allow(missing_docs)]
    pub struct Response {
        pub response_info: ResponseInfo,
        pub headers: Vec<request::Header>,
        #[pin]
        pub stream: Transport,
        pub(crate) consumed: bool,
        pub(crate) request_chunked: bool,
        pub(crate) current_chunk_size: usize,
        pub(crate) read_chunk_size: usize,
        pub(crate) chunk_parse_buffer: BytesMut,
        pub(crate) crlf_skip_done: bool,
    }
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
    #[cfg(all(feature = "async", feature = "gzip"))]
    /// Reads the full body into memory.
    pub(crate) async fn _read_to_end(&mut self) -> Result<Vec<u8>, RequestError> {
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
    /// Reads the full body into memory.
    pub(crate) fn _read_to_end(&mut self) -> Result<Vec<u8>, RequestError> {
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
    /// Reads the body as UTF-8 text, with optional gzip decoding.
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

                let read_data = self._read_to_end().await?;
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
            let mut str_buffer = Vec::new();
            self.read_to_end(&mut str_buffer).await?;

            Ok(
                String::from_utf8(str_buffer)
                    .map_err(|e| RequestError::TextError(e.to_string()))?,
            )
        }
    }

    #[cfg(not(feature = "async"))]
    /// Reads the body as UTF-8 text, with optional gzip decoding.
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
            self.read_to_end(&mut str_buffer)?;

            Ok(
                String::from_utf8(str_buffer)
                    .map_err(|e| RequestError::TextError(e.to_string()))?,
            )
        }
    }

    #[cfg(all(feature = "async", feature = "json"))]
    /// Deserializes the response body from JSON.
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, RequestError> {
        self.consumed = true;
        let string = self.text().await?;
        let json: T = serde_json::from_str(&string)?;
        Ok(json)
    }

    #[cfg(all(not(feature = "async"), feature = "json"))]
    /// Deserializes the response body from JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, RequestError> {
        self.consumed = true;
        let string = self.text()?;
        let json: T = serde_json::from_str(&string)?;
        Ok(json)
    }

    #[cfg(all(feature = "async", feature = "gzip"))]
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

        usize::from_str_radix(chunk_size_string.trim_end(), 16)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
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

        usize::from_str_radix(chunk_size_string.trim_end(), 16)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

#[cfg(not(feature = "async"))]
impl Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.request_chunked {
            if self.consumed {
                return Ok(0);
            }

            if self.current_chunk_size == 0 && self.read_chunk_size == 0 {
                self.current_chunk_size = self.read_chunk_size(false)?;

                // A body that is only the terminating zero-length chunk is
                // empty, not an error: return before the loop asks for
                // another chunk header.
                if self.current_chunk_size == 0 {
                    self.consumed = true;
                    return Ok(0);
                }
            }

            let mut read_buffer_len = 0;
            while read_buffer_len != buf.len() {
                if self.current_chunk_size == self.read_chunk_size {
                    self.current_chunk_size = self.read_chunk_size(true)?;
                    self.read_chunk_size = 0;
                    if self.current_chunk_size == 0 {
                        self.consumed = true;
                        return Ok(read_buffer_len);
                    }
                }
                let remaining_chunk_size = self.current_chunk_size - self.read_chunk_size;
                let pointer = remaining_chunk_size.min(buf.len() - read_buffer_len);
                let read_byte = self
                    .stream
                    .read(&mut buf[read_buffer_len..(pointer + read_buffer_len)])?;

                // Without this the loop would spin forever on a truncated body,
                // since read_buffer_len can never reach buf.len().
                if read_byte == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "connection closed while reading chunk data",
                    ));
                }

                self.read_chunk_size += read_byte;
                read_buffer_len += read_byte;
            }

            Ok(buf.len())
        } else {
            self.stream.read(buf)
        }
    }
}

#[cfg(feature = "async")]
impl AsyncRead for Response {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        use tokio::io::AsyncBufRead;

        let mut this = self.project();

        if !*this.request_chunked {
            return this.stream.poll_read(cx, buf);
        }

        if *this.consumed || buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        // Advance to the next chunk when the current one is exhausted.
        // current_chunk_size == read_chunk_size means we need a new chunk header.
        if *this.current_chunk_size == *this.read_chunk_size {
            // For non-initial chunks, skip the trailing \r\n that follows the
            // chunk data before reading the next chunk-size line.
            // crlf_skip_done tracks that the trailing CRLF was already consumed so a
            // Poll::Pending from the chunk-size parser doesn't re-trigger the skip.
            if *this.current_chunk_size > 0 && !*this.crlf_skip_done {
                loop {
                    let found_lf;
                    let n;
                    {
                        let available = match this.stream.as_mut().poll_fill_buf(cx) {
                            Poll::Ready(Ok(data)) => data,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Pending => return Poll::Pending,
                        };
                        if available.is_empty() {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "connection closed while skipping chunk CRLF",
                            )));
                        }
                        match available.iter().position(|&b| b == b'\n') {
                            Some(pos) => { n = pos + 1; found_lf = true; }
                            None => { n = available.len(); found_lf = false; }
                        }
                    }
                    this.stream.as_mut().consume(n);
                    if found_lf {
                        *this.crlf_skip_done = true;
                        break;
                    }
                }
            }

            // Read the chunk-size line (hex digits followed by \r\n), accumulating
            // partial data in chunk_parse_buffer across Poll::Pending returns.
            loop {
                let found_lf;
                let n;
                {
                    let available = match this.stream.as_mut().poll_fill_buf(cx) {
                        Poll::Ready(Ok(data)) => data,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => return Poll::Pending,
                    };
                    if available.is_empty() {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "connection closed while reading chunk size",
                        )));
                    }
                    match available.iter().position(|&b| b == b'\n') {
                        Some(pos) => {
                            this.chunk_parse_buffer.extend_from_slice(&available[..pos]);
                            n = pos + 1;
                            found_lf = true;
                        }
                        None => {
                            this.chunk_parse_buffer.extend_from_slice(available);
                            n = available.len();
                            found_lf = false;
                        }
                    }
                }
                this.stream.as_mut().consume(n);
                if found_lf { break; }
            }

            let hex_str = std::str::from_utf8(this.chunk_parse_buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let chunk_size = usize::from_str_radix(hex_str.trim(), 16)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            this.chunk_parse_buffer.clear();
            *this.crlf_skip_done = false;
            *this.current_chunk_size = chunk_size;
            *this.read_chunk_size = 0;

            if chunk_size == 0 {
                *this.consumed = true;
                return Poll::Ready(Ok(()));
            }
        }

        // Read data from the current chunk, capped at remaining bytes in this chunk.
        let remaining_in_chunk = *this.current_chunk_size - *this.read_chunk_size;
        let to_copy;
        {
            let available = match this.stream.as_mut().poll_fill_buf(cx) {
                Poll::Ready(Ok(data)) => data,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            };
            if available.is_empty() {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "connection closed while reading chunk data",
                )));
            }
            to_copy = remaining_in_chunk.min(buf.remaining()).min(available.len());
            buf.put_slice(&available[..to_copy]);
        }
        this.stream.as_mut().consume(to_copy);
        *this.read_chunk_size += to_copy;

        Poll::Ready(Ok(()))
    }
}
