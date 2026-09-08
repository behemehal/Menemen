use std::pin::Pin;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream};

#[cfg(feature = "https")]
use tokio_native_tls::TlsStream as TokioTlsStream;

use std::task::{Context, Poll};

use tokio::net::TcpStream;

/// This enum is a bridge for the different types of streams that can be used to communicate with the server.
#[allow(missing_debug_implementations)]
pub enum Transport {
    #[cfg(feature = "https")]
    /// TLS-wrapped stream.
    Ssl(BufStream<TokioTlsStream<TcpStream>>),
    /// Tcp stream
    Tcp(BufStream<TcpStream>),
}

impl AsyncRead for Transport {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_read(cx, buf),
            Transport::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Transport {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_write(cx, buf),
            Transport::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_flush(cx),
            Transport::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "https")]
            Transport::Ssl(stream) => Pin::new(stream).poll_shutdown(cx),
            Transport::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
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
