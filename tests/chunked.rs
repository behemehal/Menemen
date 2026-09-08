//! Response-decoder tests driven by an in-process TCP server that replays canned
//! HTTP responses. No network access and no external server are required.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

/// Serves `raw` verbatim to exactly one client, then closes the connection.
/// Returns the port it bound to.
fn serve_once(raw: &'static [u8]) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local_addr").port();

    thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            // Drain the request head so the client's write completes.
            let mut scratch = [0u8; 8192];
            let _ = socket.read(&mut scratch);
            let _ = socket.write_all(raw);
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
}
