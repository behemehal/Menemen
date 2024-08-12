use std::io::Cursor;

#[cfg(not(feature = "async"))]
use std::{
    fs::File,
    io::{Read, Write},
};

#[cfg(feature = "async")]
use std::pin::Pin;
use tokio::{fs::File as TokioFile, io::AsyncRead};

pub struct Body {
    body: BodyType,
}

impl Body {
    #[cfg(not(feature = "async"))]
    pub fn from_reader<R: Read + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    #[cfg(feature = "async")]
    pub fn from_reader<R: AsyncRead + Unpin + 'static>(reader: R) -> Body {
        Body {
            body: BodyType::Reader(Box::new(reader)),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Body {
        Body {
            body: BodyType::Bytes(bytes),
        }
    }

    pub fn size_hint(&self) -> Option<usize> {
        match &self.body {
            BodyType::Reader(reader) => None,
            BodyType::Bytes(bytes) => Some(bytes.len()),
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
    #[cfg(feature = "async")]
    Reader(Box<dyn AsyncRead + Unpin>),
    #[cfg(not(feature = "async"))]
    Reader(Box<dyn Read>),
    Bytes(Vec<u8>),
}

#[cfg(not(feature = "async"))]
impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.body.read(buf)
    }
}

#[cfg(not(feature = "async"))]
impl Read for BodyType {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            BodyType::Reader(reader) => reader.read(buf),
            BodyType::Bytes(bytes) => Cursor::new(bytes).read(buf),
        }
    }
}

#[cfg(feature = "async")]
impl AsyncRead for Body {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut pinned = Pin::new(&mut self.get_mut().body);
        pinned.as_mut().poll_read(cx, buf)
    }
}

#[cfg(feature = "async")]
impl AsyncRead for BodyType {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            BodyType::Reader(mut reader) => Pin::new(&mut reader).poll_read(cx, buf),
            BodyType::Bytes(bytes) => {
                let mut cursor = Cursor::new(bytes);
                let mut pinned = Pin::new(&mut cursor);
                pinned.as_mut().poll_read(cx, buf)
            }
        }
    }
}
//

#[cfg(not(feature = "async"))]
impl Write for Body {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.body.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "async"))]
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

#[cfg(not(feature = "async"))]
impl From<File> for Body {
    fn from(file: File) -> Body {
        Body::from_reader(file)
    }
}

#[cfg(feature = "async")]
impl From<TokioFile> for Body {
    fn from(file: TokioFile) -> Body {
        Body::from_reader(file)
    }
}
