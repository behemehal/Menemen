use openssl::ssl::SslStream;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Transport {
    Ssl(SslStream<TcpStream>),
    Tcp(TcpStream),
}

impl Transport {
    pub fn close(&mut self) {
        match self {
            Transport::Ssl(e) => {
                e.shutdown().unwrap();
            }
            Transport::Tcp(e) => {
                e.shutdown(std::net::Shutdown::Both).unwrap();
            }
        }
    }
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
