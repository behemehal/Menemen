use std::{fs::File, io::Read};

use std::io::Cursor;

use crate::form_data::FormData;

#[cfg(feature = "multipart")]
use super::multipart_form::MultipartFormData;

/// Request body container for blocking mode.
pub struct Body {
    /// Concrete body representation.
    pub body: BodyType,
}

impl Body {
    /// Creates a body from any blocking reader.
    pub fn from_reader<R: Read + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    /// Creates a body from an owned byte buffer.
    pub fn from_bytes(bytes: Vec<u8>) -> Body {
        Body {
            body: BodyType::Bytes(Cursor::new(bytes)),
        }
    }

    /// Returns the body size when it can be determined without reading streams.
    pub fn size_hint(&self) -> Option<usize> {
        match &self.body {
            BodyType::Reader(_) => None,
            BodyType::Bytes(cursor) => Some(cursor.get_ref().len()),
            #[cfg(feature = "multipart")]
            BodyType::MultipartFormData(_) => None,
            BodyType::FormData(form_data) => Some(form_data.build().into_bytes().len()),
        }
    }

    /// Returns the suggested content type for this body when known.
    pub fn content_type(&self) -> Option<String> {
        match &self.body {
            BodyType::Reader(_) => None,
            BodyType::Bytes(_) => None,
            #[cfg(feature = "multipart")]
            BodyType::MultipartFormData(_) => Some("multipart/form-data".to_string()),
            BodyType::FormData(_) => Some("application/x-www-form-urlencoded".to_string()),
        }
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            BodyType::Reader(_) => write!(f, "BodyType::Reader"),
            BodyType::Bytes(bytes) => write!(f, "BodyType::Bytes({:?})", bytes),
            #[cfg(feature = "multipart")]
            BodyType::MultipartFormData(_) => write!(f, "BodyType::MultipartFormData"),
            BodyType::FormData(_) => write!(f, "BodyType::FormData"),
        }
    }
}

/// Concrete blocking-mode body variants.
pub enum BodyType {
    /// Streaming reader content.
    Reader(Box<dyn Read>),
    /// In-memory bytes.
    Bytes(Cursor<Vec<u8>>),
    /// URL-encoded form data.
    FormData(FormData),
    #[cfg(feature = "multipart")]
    /// Multipart form-data content.
    MultipartFormData(MultipartFormData),
}

impl std::fmt::Debug for BodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyType::Reader(_) => f.write_str("BodyType::Reader"),
            BodyType::Bytes(bytes) => f.debug_tuple("BodyType::Bytes").field(bytes).finish(),
            BodyType::FormData(_) => f.write_str("BodyType::FormData"),
            #[cfg(feature = "multipart")]
            BodyType::MultipartFormData(_) => f.write_str("BodyType::MultipartFormData"),
        }
    }
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.body {
            BodyType::Reader(reader) => reader.read(buf),
            BodyType::Bytes(cursor) => cursor.read(buf),
            BodyType::FormData(form_data) => {
                let form_data_string = form_data.build();
                let mut bytes = form_data_string.into_bytes();
                bytes.extend(b"\n");
                let mut cursor = Cursor::new(bytes);
                cursor.read(buf)
            }
            #[cfg(feature = "multipart")]
            BodyType::MultipartFormData(data) => {
                let built = data
                    .build()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                let mut cursor = Cursor::new(built);
                cursor.read(buf)
            }
        }
    }
}

impl From<Cursor<Vec<u8>>> for Body {
    fn from(cursor: Cursor<Vec<u8>>) -> Body {
        Body::from_reader(cursor)
    }
}

impl From<Cursor<&'static [u8]>> for Body {
    fn from(cursor: Cursor<&'static [u8]>) -> Body {
        Body::from_reader(cursor)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Body {
        Body::from_reader(Cursor::new(s))
    }
}

impl From<Vec<u8>> for Body {
    fn from(bytes: Vec<u8>) -> Body {
        Body::from_bytes(bytes)
    }
}

impl From<&'static [u8]> for Body {
    fn from(bytes: &'static [u8]) -> Body {
        Body::from_bytes(bytes.to_vec())
    }
}

impl From<&'static str> for Body {
    fn from(s: &'static str) -> Body {
        Body::from_reader(Cursor::new(s))
    }
}

impl From<File> for Body {
    fn from(file: File) -> Body {
        Body::from_reader(file)
    }
}

impl From<FormData> for Body {
    fn from(value: FormData) -> Self {
        let built_form_data = value.build();
        let cursor = Cursor::new(built_form_data.into_bytes());
        Body {
            body: BodyType::Bytes(cursor),
        }
    }
}

#[cfg(feature = "multipart")]
impl From<MultipartFormData> for Body {
    fn from(value: MultipartFormData) -> Self {
        Body {
            body: BodyType::MultipartFormData(value),
        }
    }
}
