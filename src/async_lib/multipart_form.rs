use crate::error::RequestError;
use tokio::{fs, io::AsyncRead};

pub enum MultipartFormValue {
    Stream(Box<dyn AsyncRead + Unpin>),
    Text(String),
}

pub struct MultipartFormData {
    pub(crate) form_data: Vec<(String, MultipartFormValue)>,
    pub boundary: String,
}

impl MultipartFormData {
    pub fn new() -> MultipartFormData {
        MultipartFormData {
            form_data: Vec::new(),
            boundary: "boundary".to_string(),
        }
    }

    pub fn add_string(&mut self, name: &str, data: String) {
        self.form_data
            .push((name.to_string(), MultipartFormValue::Text(data)));
    }

    pub async fn add_file(&mut self, name: &str, file_path: &str) -> Result<(), RequestError> {
        //Check if file exists
        let attr = fs::metadata(file_path).await;

        if attr.is_err() {
            return Err(RequestError::FileError("File not found".to_string()));
        }

        let file = fs::File::open(file_path).await?;
        self.form_data
            .push((name.to_string(), MultipartFormValue::Stream(Box::new(file))));
        Ok(())
    }

    pub fn add_stream<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        self.form_data
            .push((name.to_string(), MultipartFormValue::Stream(Box::new(data))));
    }

    pub fn set_string(&mut self, name: &str, data: String) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add_string(name, data);
    }

    pub fn set_stream<T: AsyncRead + Unpin + 'static>(&mut self, name: &str, data: T) {
        let key_exists = self.form_data.iter().any(|pair| pair.0 == name);
        if key_exists {
            self.remove(name);
        }
        self.add_stream(name, data);
    }

    pub fn remove(&mut self, name: &str) {
        self.form_data.retain(|pair| pair.0 != name);
    }

    #[cfg(feature = "async")]
    pub(crate) async fn build(&mut self) -> Result<Vec<u8>, RequestError> {
        use tokio::io::AsyncReadExt;

        let mut byte_buffer = Vec::new();

        for (name, data) in self.form_data.iter_mut() {
            byte_buffer.extend_from_slice(format!("--{}\r\n", self.boundary).as_bytes());
            byte_buffer.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
            );

            let mut buffer = Vec::new();
            match data {
                MultipartFormValue::Stream(stream) => {
                    stream.read_to_end(&mut buffer).await.unwrap();
                }
                MultipartFormValue::Text(text) => {
                    buffer.extend_from_slice(text.as_bytes());
                }
            }
            byte_buffer.extend_from_slice(&buffer);
            byte_buffer.extend_from_slice(b"\r\n");
        }

        byte_buffer.extend_from_slice(format!("--{}--\r\n", self.boundary).as_bytes());
        Ok(byte_buffer)
    }
}
