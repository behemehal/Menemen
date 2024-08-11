use std::{
    fs::File,
    io::{Cursor, Read, Write},
};

pub struct Body {
    body: BodyType,
}

impl Body {
    pub fn from_reader<R: Read + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Body {
        Body {
            body: BodyType::Bytes(bytes),
        }
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            BodyType::Reader(_) => write!(f, "BodyType::Reader"),
            BodyType::Bytes(bytes) => write!(f, "BodyType::Bytes({:?})", bytes),
        }
    }
}

pub enum BodyType {
    Reader(Box<dyn Read>),
    Bytes(Vec<u8>),
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.body.read(buf)
    }
}

impl Read for BodyType {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            BodyType::Reader(reader) => reader.read(buf),
            BodyType::Bytes(bytes) => Cursor::new(bytes).read(buf),
        }
    }
}

impl Write for Body {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.body.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
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
                bytes.extend_from_slice(buf);
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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
