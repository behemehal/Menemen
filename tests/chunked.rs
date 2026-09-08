//! Response-decoder tests driven by an in-process TCP server that replays canned
//! HTTP responses. No network access and no external server are required.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

/// Serves `raw` verbatim to exactly one client, then closes the connection.
/// Returns the port it bound to.
fn serve_once(raw: &'static [u8]) -> u16 {
    serve_bytes(raw.to_vec())
}

/// [`serve_once`] for a response assembled at runtime.
fn serve_bytes(raw: Vec<u8>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local_addr").port();

    thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            // Drain the request head so the client's write completes.
            let mut scratch = [0u8; 8192];
            let _ = socket.read(&mut scratch);
            let _ = socket.write_all(&raw);
            let _ = socket.flush();
        }
    });

    port
}

fn url_for(port: u16) -> String {
    format!("http://127.0.0.1:{}/", port)
}

const EMPTY_CHUNKED: &[u8] = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n";

const SINGLE_CHUNK: &[u8] =
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nc\r\nHello, World\r\n0\r\n\r\n";

const MULTI_CHUNK: &[u8] = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\
5\r\nHello\r\n\
2\r\n, \r\n\
6\r\nWorld!\r\n\
0\r\n\r\n";

/// 0x1a == 26 bytes, so this also covers lower-case hex chunk sizes.
const HEX_CHUNK: &[u8] = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\
1a\r\nabcdefghijklmnopqrstuvwxyz\r\n\
0\r\n\r\n";

const CONTENT_LENGTH: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, World";

const NOT_FOUND_CHUNKED: &[u8] =
    b"HTTP/1.1 404 Not Found\r\nTransfer-Encoding: chunked\r\n\r\n9\r\nnot here!\r\n0\r\n\r\n";

/// Announces a 10-byte chunk, sends 4 bytes, then closes.
const TRUNCATED_CHUNK: &[u8] =
    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\na\r\nabcd";

/// gzip of "hello gzip from menemen".
#[cfg(feature = "gzip")]
const GZIP_BODY: &[u8] = &[
    0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57,
    0x48, 0xaf, 0xca, 0x2c, 0x50, 0x48, 0x2b, 0xca, 0xcf, 0x55, 0xc8, 0x4d, 0xcd, 0x4b, 0xcd, 0x4d,
    0xcd, 0x03, 0x00, 0x0f, 0x2f, 0x6e, 0xcb, 0x17, 0x00, 0x00, 0x00,
];

#[cfg(feature = "gzip")]
fn gzip_response() -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
        GZIP_BODY.len()
    )
    .into_bytes();
    response.extend_from_slice(GZIP_BODY);
    response
}

/// Same payload, delivered with chunked framing instead of Content-Length.
#[cfg(feature = "gzip")]
fn gzip_chunked_response() -> Vec<u8> {
    let mut response =
        b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
    response.extend_from_slice(format!("{:x}\r\n", GZIP_BODY.len()).as_bytes());
    response.extend_from_slice(GZIP_BODY);
    response.extend_from_slice(b"\r\n0\r\n\r\n");
    response
}

#[cfg(not(feature = "async"))]
mod blocking_decoder {
    use super::*;
    use menemen::request::{Request, RequestTypes};

    fn get(port: u16) -> menemen::response::Response {
        let mut request = Request::new(&url_for(port), RequestTypes::GET).expect("build request");
        request.send().expect("send request")
    }

    /// Regression: a body consisting only of the terminating zero-length chunk
    /// used to fail with "Connection closed by server".
    #[test]
    fn empty_chunked_body_is_not_an_error() {
        let mut response = get(serve_once(EMPTY_CHUNKED));
        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().expect("decode"), "");
    }

    #[test]
    fn single_chunk_body() {
        let mut response = get(serve_once(SINGLE_CHUNK));
        assert_eq!(response.text().expect("decode"), "Hello, World");
    }

    #[test]
    fn multiple_chunks_are_concatenated() {
        let mut response = get(serve_once(MULTI_CHUNK));
        assert_eq!(response.text().expect("decode"), "Hello, World!");
    }

    #[test]
    fn lowercase_hex_chunk_size() {
        let mut response = get(serve_once(HEX_CHUNK));
        assert_eq!(response.text().expect("decode"), "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn content_length_body() {
        let mut response = get(serve_once(CONTENT_LENGTH));
        assert_eq!(response.text().expect("decode"), "Hello, World");
    }

    #[test]
    fn status_and_headers_are_parsed() {
        let response = get(serve_once(NOT_FOUND_CHUNKED));
        assert_eq!(response.response_info.status_code, 404);
        assert_eq!(response.response_info.status_message, "Not Found");
        assert!(response
            .headers
            .iter()
            .any(|h| h.name == "Transfer-Encoding" && h.value == "chunked"));
    }

    /// The `Read` impl must decode chunks transparently, not hand back the
    /// chunk-size framing.
    #[test]
    fn read_trait_decodes_chunks() {
        use std::io::Read;

        let mut response = get(serve_once(MULTI_CHUNK));
        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).expect("read_to_end");
        assert_eq!(String::from_utf8(buffer).expect("utf8"), "Hello, World!");
    }

    /// Regression: a chunk that announces more bytes than the server delivers
    /// used to spin forever, because a zero-length read never advanced the
    /// fill counter. Run on a worker thread so a regression fails on the
    /// timeout rather than hanging the whole suite.
    #[test]
    fn truncated_chunk_errors_instead_of_spinning() {
        use std::sync::mpsc;
        use std::time::Duration;

        let port = serve_once(TRUNCATED_CHUNK);
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let outcome = Request::new(&url_for(port), RequestTypes::GET)
                .expect("build request")
                .send()
                .and_then(|mut response| response.text());
            let _ = sender.send(outcome.is_err());
        });

        match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(errored) => assert!(
                errored,
                "a truncated chunked body should surface an error, not succeed"
            ),
            Err(_) => panic!("timed out: the chunked reader is spinning on a truncated body"),
        }
    }

    #[cfg(feature = "gzip")]
    #[test]
    fn gzip_body_is_decoded() {
        let mut response = get(serve_bytes(gzip_response()));
        assert_eq!(
            response.text().expect("decode gzip"),
            "hello gzip from menemen"
        );
    }

    #[cfg(feature = "gzip")]
    #[test]
    fn gzip_over_chunked_is_decoded() {
        let mut response = get(serve_bytes(gzip_chunked_response()));
        assert_eq!(
            response.text().expect("decode chunked gzip"),
            "hello gzip from menemen"
        );
    }
}

#[cfg(feature = "async")]
mod async_decoder {
    use super::*;
    use menemen::request::{Request, RequestTypes};

    async fn get(port: u16) -> menemen::response::Response {
        let mut request = Request::new(&url_for(port), RequestTypes::GET).expect("build request");
        request.send().await.expect("send request")
    }

    /// Regression guard for the same empty-body case on the async path.
    #[tokio::test]
    async fn empty_chunked_body_is_not_an_error() {
        let mut response = get(serve_once(EMPTY_CHUNKED)).await;
        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().await.expect("decode"), "");
    }

    #[tokio::test]
    async fn single_chunk_body() {
        let mut response = get(serve_once(SINGLE_CHUNK)).await;
        assert_eq!(response.text().await.expect("decode"), "Hello, World");
    }

    #[tokio::test]
    async fn multiple_chunks_are_concatenated() {
        let mut response = get(serve_once(MULTI_CHUNK)).await;
        assert_eq!(response.text().await.expect("decode"), "Hello, World!");
    }

    #[tokio::test]
    async fn lowercase_hex_chunk_size() {
        let mut response = get(serve_once(HEX_CHUNK)).await;
        assert_eq!(
            response.text().await.expect("decode"),
            "abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[tokio::test]
    async fn content_length_body() {
        let mut response = get(serve_once(CONTENT_LENGTH)).await;
        assert_eq!(response.text().await.expect("decode"), "Hello, World");
    }

    #[tokio::test]
    async fn status_and_headers_are_parsed() {
        let response = get(serve_once(NOT_FOUND_CHUNKED)).await;
        assert_eq!(response.response_info.status_code, 404);
        assert_eq!(response.response_info.status_message, "Not Found");
        assert!(response
            .headers
            .iter()
            .any(|h| h.name == "Transfer-Encoding" && h.value == "chunked"));
    }

    /// The `AsyncRead` impl must decode chunks transparently.
    #[tokio::test]
    async fn async_read_trait_decodes_chunks() {
        use tokio::io::AsyncReadExt;

        let mut response = get(serve_once(MULTI_CHUNK)).await;
        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer).await.expect("read_to_end");
        assert_eq!(String::from_utf8(buffer).expect("utf8"), "Hello, World!");
    }

    #[tokio::test]
    async fn truncated_chunk_is_an_error() {
        let mut response = get(serve_once(TRUNCATED_CHUNK)).await;
        assert!(
            response.text().await.is_err(),
            "a truncated chunked body should surface an error"
        );
    }

    /// Regression: async gzip decoding never compiled, so this path was
    /// entirely unexercised.
    #[cfg(feature = "gzip")]
    #[tokio::test]
    async fn gzip_body_is_decoded() {
        let mut response = get(serve_bytes(gzip_response())).await;
        assert_eq!(
            response.text().await.expect("decode gzip"),
            "hello gzip from menemen"
        );
    }

    #[cfg(feature = "gzip")]
    #[tokio::test]
    async fn gzip_over_chunked_is_decoded() {
        let mut response = get(serve_bytes(gzip_chunked_response())).await;
        assert_eq!(
            response.text().await.expect("decode chunked gzip"),
            "hello gzip from menemen"
        );
    }
}
