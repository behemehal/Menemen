#[cfg(feature = "https")]
use native_tls::TlsStream;

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

/// This enum is a bridge for the different types of streams that can be used to communicate with the server.
///
/// Reads are buffered because header and chunk-size parsing is line-oriented.
/// Writes go straight to the socket: a request is emitted as one head write plus
/// at most one body write, so an extra write buffer would only add a copy.
#[allow(missing_debug_implementations)]
pub enum Transport {
    /// Ssl stream
    #[cfg(feature = "https")]
    Ssl(BufReader<TlsStream<TcpStream>>),
    /// Tcp stream
    Tcp(BufReader<TcpStream>),
}

impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.get_mut().write(buf),
            Transport::Tcp(socket) => socket.get_mut().write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.get_mut().flush(),
            Transport::Tcp(socket) => socket.get_mut().flush(),
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "https")]
            Transport::Ssl(socket) => socket.read(buf),
            Transport::Tcp(socket) => socket.read(buf),
        }
    }
}

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
