use std::io::Cursor;

use crate::form_data::FormData;

use tokio::{fs::File as TokioFile, io::AsyncRead};

#[cfg(feature = "multipart")]
use super::multipart_form::MultipartFormData;

/// Request body container for async mode.
pub struct Body {
    /// Concrete body representation.
    pub body: BodyType,
}

impl Body {
    /// Creates a body from any async reader.
    pub fn from_reader<R: AsyncRead + Unpin + 'static>(reader: R) -> Body {
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

/// Concrete async-mode body variants.
pub enum BodyType {
    /// Streaming async reader content.
    Reader(Box<dyn AsyncRead + Unpin>),
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

impl Into<Body> for Cursor<Vec<u8>> {
    fn into(self) -> Body {
        Body::from_reader(self)
    }
}

impl Into<Body> for Cursor<&'static [u8]> {
    fn into(self) -> Body {
        Body::from_reader(self)
    }
}

impl Into<Body> for Cursor<String> {
    fn into(self) -> Body {
        Body::from_reader(self)
    }
}

impl Into<Body> for Cursor<&'static str> {
    fn into(self) -> Body {
        Body::from_reader(self)
    }
}

#[cfg(feature = "multipart")]
impl TryFrom<Body> for MultipartFormData {
    type Error = Body;

    /// Recovers the multipart builder from a body.
    ///
    /// Returns the body unchanged when it holds something else, rather than
    /// panicking as the previous `Into` implementation did.
    fn try_from(body: Body) -> Result<Self, Self::Error> {
        match body.body {
            BodyType::MultipartFormData(multipart_form_data) => Ok(multipart_form_data),
            other => Err(Body { body: other }),
        }
    }
}
//

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

impl From<TokioFile> for Body {
    fn from(file: TokioFile) -> Body {
        Body::from_reader(file)
    }
}

impl From<FormData> for Body {
    fn from(value: FormData) -> Self {
        // Kept as FormData rather than flattened to bytes so the client can set
        // Content-Type: application/x-www-form-urlencoded for it.
        Body {
            body: BodyType::FormData(value),
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
