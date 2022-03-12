use bufstream::BufStream;
use native_tls::TlsStream;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

/// This enum is a bridge for the different types of streams that can be used to communicate with the server.
#[allow(missing_debug_implementations)]
pub enum Transport {
    /// Ssl stream
    Ssl(BufStream<TlsStream<TcpStream>>),
    /// Tcp stream
    Tcp(BufStream<TcpStream>),
}

impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Transport::Ssl(socket) => socket.write(buf),
            Transport::Tcp(socket) => socket.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Transport::Ssl(socket) => socket.flush(),
            Transport::Tcp(socket) => socket.flush(),
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Transport::Ssl(socket) => socket.read(buf),
            Transport::Tcp(socket) => socket.read(buf),
        }
    }
}

impl std::io::BufRead for Transport {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Transport::Ssl(socket) => socket.fill_buf(),
            Transport::Tcp(socket) => socket.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Transport::Ssl(socket) => socket.consume(amt),
            Transport::Tcp(socket) => socket.consume(amt),
        }
    }
}
