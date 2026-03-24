use crate::error::RequestError;
use rand::prelude::*;
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
};

/// Multipart file part metadata and reader.
pub struct MultipartFile {
    /// File reader used to stream data into the multipart payload.
    pub file: Box<dyn AsyncRead + Unpin>,
    /// File name sent in `Content-Disposition`.
    pub file_name: String,
    /// MIME type sent for this file part.
    pub content_type: String,
}

impl std::fmt::Debug for MultipartFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultipartFile")
            .field("file_name", &self.file_name)
            .field("content_type", &self.content_type)
            .finish()
    }
}

/// Multipart form value variant.
pub enum MultipartFormValue {
    /// File part.
    File(MultipartFile),
    /// Generic async stream part.
    Stream(Box<dyn AsyncRead + Unpin>),
    /// Text part.
    Text(String),
}

impl std::fmt::Debug for MultipartFormValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartFormValue::File(file) => f.debug_tuple("File").field(file).finish(),
            MultipartFormValue::Stream(_) => f.write_str("Stream(<async-reader>)"),
            MultipartFormValue::Text(text) => f.debug_tuple("Text").field(text).finish(),
        }
    }
}

/// Multipart form-data builder for async mode.
pub struct MultipartFormData {
    pub(crate) form_data: Vec<(String, MultipartFormValue)>,
    pub(crate) boundary: String,
}

impl std::fmt::Debug for MultipartFormData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultipartFormData")
            .field("parts", &self.form_data.len())
            .field("boundary", &self.boundary)
            .finish()
    }
}

impl MultipartFormData {
    /// Creates an empty multipart form-data builder with a random boundary.
    pub fn new() -> MultipartFormData {
        let mut rng = rand::rng();
        let boundary: String = (0..30)
            .map(|_| {
                let c: char = rng.random_range(48..122).into();
                c
            })
            .collect();
        MultipartFormData {
            form_data: Vec::new(),
            boundary: format!("----{}", boundary),
        }
    }

    /// Adds a text part.
    pub fn add_string(&mut self, name: &str, data: String) {
        self.form_data
            .push((name.to_string(), MultipartFormValue::Text(data)));
    }

    /// Adds a file part by opening a file path.
    pub async fn add_file(&mut self, name: &str, file_path: &str) -> Result<(), RequestError> {
        //Check if file exists
        if fs::metadata(file_path).await.is_err() {
            return Err(RequestError::FileError("File not found".to_string()));
        };

        let content_type = mime_guess::from_path(file_path)
            .first_or_text_plain()
            .to_string();

        let file = fs::File::open(file_path).await?;
        let file_name = file_path.split('/').last().unwrap().to_string();

        let file = MultipartFile {
            file: Box::new(file),
            file_name,
            content_type,
        };
        self.form_data
            .push((name.to_string(), MultipartFormValue::File(file)));
        Ok(())
    }

    /// Adds an async stream part.
    pub fn add_stream<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        self.form_data
            .push((name.to_string(), MultipartFormValue::Stream(Box::new(data))));
    }

    /// Sets a text part value, replacing previous values for the same key.
    pub fn set_string(&mut self, name: &str, data: String) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add_string(name, data);
    }

    /// Sets a stream part value, replacing previous values for the same key.
    pub fn set_stream<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add_stream(name, data);
    }

    /// Removes all parts with the provided field name.
    pub fn remove(&mut self, name: &str) {
        self.form_data.retain(|pair| pair.0 != name);
    }

    /// Builds the full multipart request body bytes.
    pub async fn build(&mut self) -> Result<Vec<u8>, RequestError> {
        let mut byte_buffer = Vec::new();

        for (name, data) in self.form_data.iter_mut() {
            byte_buffer.extend_from_slice(format!("--{}", self.boundary).as_bytes());

            let mut buffer = Vec::new();

            match data {
                MultipartFormValue::Stream(stream) => {
                    byte_buffer.extend_from_slice(
                        format!("\r\nContent-Disposition: form-data; name=\"{}\";\r\n", name)
                            .as_bytes(),
                    );
                    byte_buffer.extend_from_slice(
                        "Content-Type: application/octet-stream\r\n\r\n".as_bytes(),
                    );
                    stream.read_to_end(&mut buffer).await?;
                }
                MultipartFormValue::Text(text) => {
                    byte_buffer.extend_from_slice(
                        format!(
                            "\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n",
                            name
                        )
                        .as_bytes(),
                    );
                    buffer.extend_from_slice(text.as_bytes());
                }
                MultipartFormValue::File(file) => {
                    byte_buffer.extend_from_slice(
                        format!(
                            "\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                            name, file.file_name
                        )
                        .as_bytes(),
                    );
                    byte_buffer.extend_from_slice(
                        format!("Content-Type: {}\r\n\r\n", file.content_type).as_bytes(),
                    );
                    file.file.read_to_end(&mut buffer).await?;
                }
            }
            byte_buffer.extend_from_slice(&buffer);
            byte_buffer.extend_from_slice(b"\r\n");
        }

        byte_buffer.extend_from_slice(format!("--{}--\r\n", self.boundary).as_bytes());
        Ok(byte_buffer)
    }
}
