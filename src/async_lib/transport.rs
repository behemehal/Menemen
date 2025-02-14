use std::pin::Pin;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream};

#[cfg(feature = "https")]
use native_tls::TlsStream;

#[cfg(feature = "https")]
use tokio_native_tls::TlsStream as TokioTlsStream;

use std::task::{Context, Poll};

use tokio::net::TcpStream;

/// This enum is a bridge for the different types of streams that can be used to communicate with the server.
#[allow(missing_debug_implementations)]
pub enum Transport {
    #[cfg(feature = "https")]
    Ssl(BufStream<TlsStream<TcpStream>>),
    /// Tcp stream
    Tcp(BufStream<TcpStream>),
}

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
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_shutdown(cx),
            Transport::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

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

impl AsyncBufRead for Transport {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        match self.get_mut() {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_fill_buf(cx),
            Transport::Tcp(stream) => Pin::new(stream).poll_fill_buf(cx),
        }
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        match &mut *self {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).consume(amt),
            Transport::Tcp(socket) => Pin::new(socket).consume(amt),
        }
    }
}
