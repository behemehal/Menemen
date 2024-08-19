#[cfg(not(feature = "async"))]
use std::io::Read;

#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::RequestError;

pub struct MultipartFormData {
    #[cfg(not(feature = "async"))]
    pub(crate) form_data: Vec<(String, Box<dyn Read>)>,
    #[cfg(feature = "async")]
    pub(crate) form_data: Vec<(String, Box<dyn AsyncRead + Unpin>)>,
    pub boundary: String,
}

impl MultipartFormData {
    pub fn new() -> MultipartFormData {
        MultipartFormData {
            form_data: Vec::new(),
            boundary: "boundary".to_string(),
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn add<T: Read + 'static>(&mut self, name: &str, data: T) {
        self.form_data.push((name.to_string(), Box::new(data)));
    }

    #[cfg(feature = "async")]
    pub fn add<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        self.form_data.push((name.to_string(), Box::new(data)));
    }

    #[cfg(not(feature = "async"))]
    pub fn set<T: Read + 'static>(&mut self, name: &str, data: T) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add(name, data);
    }

    #[cfg(feature = "async")]
    pub fn set<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add(name, data);
    }

    pub fn remove(&mut self, name: &str) {
        self.form_data.retain(|pair| pair.0 != name);
    }

    #[cfg(not(feature = "async"))]
    pub fn get(&self, name: &str) -> Option<&(String, Box<dyn Read>)> {
        for pair in self.form_data.iter() {
            if pair.0 == name {
                return Some(pair);
            }
        }
        None
    }

    #[cfg(feature = "async")]
    pub fn get(&self, name: &str) -> Option<&(String, Box<dyn AsyncRead + Unpin>)> {
        for pair in self.form_data.iter() {
            if pair.0 == name {
                return Some(pair);
            }
        }
        None
    }

    #[cfg(not(feature = "async"))]
    pub(crate) fn build(mut self, boundary: String) -> Result<Vec<u8>, RequestError> {
        let mut byte_buffer = Vec::new();

        for (name, data) in self.form_data.iter_mut() {
            byte_buffer.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            byte_buffer.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
            );

            let mut buffer = Vec::new();
            data.read_to_end(&mut buffer)?;
            byte_buffer.extend_from_slice(&buffer);
            byte_buffer.extend_from_slice(b"\r\n");
        }

        byte_buffer.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        Ok(byte_buffer)
    }

    #[cfg(feature = "async")]
    pub(crate) async fn build(mut self) -> Result<Vec<u8>, RequestError> {
        let mut byte_buffer = Vec::new();

        for (name, data) in self.form_data.iter_mut() {
            byte_buffer.extend_from_slice(format!("--{}\r\n", self.boundary).as_bytes());
            byte_buffer.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
            );

            let mut buffer = Vec::new();
            data.read_to_end(&mut buffer).await.unwrap();
            byte_buffer.extend_from_slice(&buffer);
            byte_buffer.extend_from_slice(b"\r\n");
        }

        byte_buffer.extend_from_slice(format!("--{}--\r\n", self.boundary).as_bytes());
        Ok(byte_buffer)
    }
}
