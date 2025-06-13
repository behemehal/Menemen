use std::{fs::File, io::Read};

use std::io::Cursor;

use crate::form_data::FormData;

#[cfg(feature = "multipart")]
use super::multipart_form::MultipartFormData;

pub struct Body {
    pub body: BodyType,
}

impl Body {
    pub fn from_reader<R: Read + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Body {
        Body {
            body: BodyType::Bytes(Cursor::new(bytes)),
        }
    }

    pub fn size_hint(&self) -> Option<usize> {
        match &self.body {
            BodyType::Reader(_) => None,
            BodyType::Bytes(cursor) => Some(cursor.get_ref().len()),
            BodyType::MultipartFormData(_) => None,
            BodyType::FormData(form_data) => Some(form_data.build().into_bytes().len()),
        }
    }

    pub fn content_type(&self) -> Option<String> {
        match &self.body {
            BodyType::Reader(_) => None,
            BodyType::Bytes(_) => None,
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
            BodyType::MultipartFormData(_) => write!(f, "BodyType::MultipartFormData"),
            BodyType::FormData(_) => write!(f, "BodyType::FormData"),
        }
    }
}

pub enum BodyType {
    Reader(Box<dyn Read>),
    Bytes(Cursor<Vec<u8>>),
    FormData(FormData),
    #[cfg(feature = "multipart")]
    MultipartFormData(MultipartFormData),
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
            BodyType::MultipartFormData(data) => {
                let built = data.build().unwrap();

                let mut cursor = Cursor::new(built);
                cursor.read(buf)
            }
        }
    }
}

/* impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.body.read(buf)
    }
}

impl Read for BodyType {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            BodyType::Reader(reader) => reader.read(buf),
            BodyType::Bytes(cursor) => {
                println!("Reading from cursor: {:#?}", cursor);

                cursor.read(buf)
            }
            BodyType::FormData(form_data) => {
                let form_data_string = form_data.build();
                println!("form_data_string: {:#?}", form_data_string);

                let mut bytes = form_data_string.into_bytes();
                bytes.extend(b"\n");

                println!("bytes: {:#?}", bytes);

                let mut cursor = Cursor::new(bytes);
                cursor.read(buf)
            }
            BodyType::MultipartFormData(_) => {
                unimplemented!()
            }
        }
    }
}
//

impl Write for Body {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unimplemented!();
        self.body.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl Write for BodyType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            BodyType::Reader(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot write to a reader",
            )),
            BodyType::Bytes(bytes) => {
                unimplemented!();
                //bytes.extend_from_slice(buf);
                //Ok(buf.len())
            }
            BodyType::FormData(form_data) => {
                unimplemented!();
                //let mut bytes = form_data.build().into_bytes();
                //bytes.extend_from_slice(buf);
                Ok(buf.len())
            }
            BodyType::MultipartFormData(_) => todo!(),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
 */
//
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

impl From<MultipartFormData> for Body {
    fn from(value: MultipartFormData) -> Self {
        Body {
            body: BodyType::MultipartFormData(value),
        }
    }
}
