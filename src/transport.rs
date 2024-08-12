#[cfg(not(feature = "async"))]
use bufstream::BufStream;

#[cfg(feature = "async")]
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, BufStream};

#[cfg(all(feature = "https", not(feature = "async")))]
use native_tls::TlsStream;

#[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
use tokio_native_tls::TlsStream;


#[cfg(not(feature = "async"))]
use std::{
    io::{BufRead, Read, Write},
    net::TcpStream,
};

#[cfg(feature = "async")]
use tokio::net::TcpStream;

/// This enum is a bridge for the different types of streams that can be used to communicate with the server.
#[allow(missing_debug_implementations)]
pub enum Transport {
    #[cfg(all(feature = "https", not(feature = "async")))]
    /// Blocking Ssl stream
    Ssl(BufStream<TlsStream<TcpStream>>),
    #[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
    Ssl(BufStream<TlsStream<TcpStream>>),
    /// Tcp stream
    Tcp(BufStream<TcpStream>),
}

#[cfg(not(feature = "async"))]
impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.write(buf),
            Transport::Tcp(socket) => socket.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.flush(),
            Transport::Tcp(socket) => socket.flush(),
        }
    }
}

#[cfg(feature = "async")]
impl AsyncWrite for Transport {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match &mut *self {
            #[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
            Transport::Ssl(stream) => Pin::new(stream).poll_write(cx, buf),

            Transport::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut *self {
            #[cfg(all(feature = "https", not(feature = "async")))]
            Transport::Ssl(stream) => Pin::new(stream).poll_flush(cx),

            #[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
            Transport::Ssl(stream) => Pin::new(stream).poll_flush(cx),

            Transport::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut *self {
            #[cfg(all(feature = "https", feature = "async", feature = "https-async"))]
            Transport::Ssl(stream) => Pin::new(stream).poll_shutdown(cx),

            Transport::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

#[cfg(not(feature = "async"))]
impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.read(buf),
            Transport::Tcp(socket) => socket.read(buf),
        }
    }
}

#[cfg(feature = "async")]
impl AsyncRead for Transport {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_read(cx, buf),
            Transport::Tcp(socket) => Pin::new(socket).poll_read(cx, buf),
        }
    }
}

#[cfg(not(feature = "async"))]
impl BufRead for Transport {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.fill_buf(),
            Transport::Tcp(socket) => socket.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.consume(amt),
            Transport::Tcp(socket) => socket.consume(amt),
        }
    }
}
