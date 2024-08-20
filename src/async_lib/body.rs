use std::io::Cursor;

use crate::form_data::FormData;

use tokio::{fs::File as TokioFile, io::AsyncRead};

#[cfg(feature = "multipart")]
use super::multipart_form::MultipartFormData;

pub struct Body {
    pub body: BodyType,
}

impl Body {
    pub fn from_reader<R: AsyncRead + Unpin + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Body {
        Body {
            body: BodyType::Bytes(Cursor::new(bytes)),
        }
    }

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

pub enum BodyType {
    Reader(Box<dyn AsyncRead + Unpin>),
    Bytes(Cursor<Vec<u8>>),
    FormData(FormData),
    #[cfg(feature = "multipart")]
    MultipartFormData(MultipartFormData),
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
impl Into<MultipartFormData> for Body {
    fn into(self) -> MultipartFormData {
        match self.body {
            BodyType::MultipartFormData(multipart_form_data) => multipart_form_data,
            _ => panic!("Body is not MultipartFormData"),
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
