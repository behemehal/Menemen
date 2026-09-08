//! Asserts the exact bytes Menemen puts on the wire.
//!
//! These run against an in-process TCP server, so no network access is needed.
//! Request construction bugs (a bogus `Content-Type`, an unescaped form value,
//! a leaked URL fragment) are invisible to response-level tests, which is how
//! several of them survived for so long.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

const OK_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";

/// Accepts one connection, reads the complete request (head plus any
/// `Content-Length` body), replies with `response`, and returns the raw request.
fn serve_and_capture(response: &'static [u8]) -> (u16, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local_addr").port();

    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept");
        let mut received: Vec<u8> = Vec::new();
        let mut scratch = [0u8; 4096];

        loop {
            // Stop once the head is complete and the advertised body has arrived.
            if let Some(head_end) = find_head_end(&received) {
                let body_len = content_length(&received[..head_end]).unwrap_or(0);
                if received.len() >= head_end + body_len {
                    break;
                }
            }

            match socket.read(&mut scratch) {
                Ok(0) => break,
                Ok(n) => received.extend_from_slice(&scratch[..n]),
                Err(_) => break,
            }
        }

        let _ = socket.write_all(response);
        let _ = socket.flush();

        String::from_utf8_lossy(&received).to_string()
    });

    (port, handle)
}

/// Serves each response in turn, one per connection, so a redirect chain can be
/// exercised. The client opens a fresh connection per request because it sends
/// `Connection: close`.
fn serve_sequence(responses: Vec<&'static [u8]>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local_addr").port();

    thread::spawn(move || {
        for response in responses {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    let mut scratch = [0u8; 8192];
                    let _ = socket.read(&mut scratch);
                    let _ = socket.write_all(response);
                    let _ = socket.flush();
                }
                Err(_) => break,
            }
        }
    });

    port
}

const REDIRECT_RESPONSE: &[u8] =
    b"HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\n\r\n";

const FINAL_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nfinal";

fn find_head_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

fn content_length(head: &[u8]) -> Option<usize> {
    let head = String::from_utf8_lossy(head);
    // The request line carries no colon, so a `?` here would bail out before
    // ever reaching the headers, and the body would never be waited for.
    for line in head.split("\r\n") {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                return value.trim().parse().ok();
            }
        }
    }
    None
}

/// Splits a captured request into its request line and its header lines.
fn head_of(raw: &str) -> (String, Vec<String>) {
    let head = raw.split("\r\n\r\n").next().unwrap_or("");
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or("").to_string();
    (request_line, lines.map(|line| line.to_string()).collect())
}

fn header_value(headers: &[String], name: &str) -> Option<String> {
    for line in headers {
        if let Some((header_name, value)) = line.split_once(':') {
            if header_name.trim().eq_ignore_ascii_case(name) {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

fn body_of(raw: &str) -> String {
    match raw.split_once("\r\n\r\n") {
        Some((_, body)) => body.to_string(),
        None => String::new(),
    }
}

#[cfg(not(feature = "async"))]
mod blocking_wire {
    use super::*;
    use menemen::request::{Request, RequestTypes};

    fn get_raw(path: &str) -> String {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}{}", port, path);
        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let _ = request.send();
        handle.join().expect("capture thread")
    }

    /// A request with no body must not describe one. The default content type
    /// used to be an `Accept`-style value emitted as `Content-Type`.
    #[test]
    fn bodyless_get_sends_no_content_type() {
        let raw = get_raw("/api");
        let (_, headers) = head_of(&raw);
        assert_eq!(
            header_value(&headers, "Content-Type"),
            None,
            "bodyless GET should not send Content-Type, got: {raw}"
        );
    }

    #[test]
    fn get_advertises_accept() {
        let raw = get_raw("/api");
        let (_, headers) = head_of(&raw);
        assert_eq!(header_value(&headers, "Accept").as_deref(), Some("*/*"));
    }

    #[test]
    fn host_header_includes_the_port() {
        let raw = get_raw("/api");
        let (_, headers) = head_of(&raw);
        let host = header_value(&headers, "Host").expect("Host header");
        assert!(host.starts_with("127.0.0.1:"), "unexpected Host: {host}");
    }

    /// A fragment is client-side only and must never be transmitted.
    #[test]
    fn url_fragment_is_not_sent() {
        let raw = get_raw("/page#section");
        let (request_line, _) = head_of(&raw);
        assert!(
            !request_line.contains('#'),
            "fragment leaked into the request line: {request_line}"
        );
        assert!(request_line.starts_with("GET /page "), "{request_line}");
    }

    /// A query string with no path used to be folded into the hostname and lost.
    #[test]
    fn query_without_a_path_is_sent() {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}?q=1&r=2", port);
        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let _ = request.send();
        let raw = handle.join().expect("capture thread");

        let (request_line, _) = head_of(&raw);
        assert!(
            request_line.contains("q=1&r=2"),
            "query string was dropped: {request_line}"
        );
    }

    /// Pre-encoded query values must pass through untouched, not be re-encoded.
    #[test]
    fn encoded_query_is_not_double_encoded() {
        let raw = get_raw("/api?q=hello%20world");
        let (request_line, _) = head_of(&raw);
        assert!(request_line.contains("q=hello%20world"), "{request_line}");
        assert!(!request_line.contains("%2520"), "{request_line}");
    }

    /// Regression: an unescaped `&` or `=` in a value used to split one field
    /// into several on the server.
    #[test]
    fn form_values_are_percent_encoded() {
        use menemen::form_data::FormData;

        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}/submit", port);

        let mut form = FormData::new();
        form.add("note", "a&b=c");
        form.add("msg", "hello world");

        let mut request = Request::new(&url, RequestTypes::POST).expect("build request");
        request.append_body(form.into());
        let _ = request.send();

        let raw = handle.join().expect("capture thread");
        assert_eq!(body_of(&raw), "note=a%26b%3Dc&msg=hello+world");
    }

    #[test]
    fn form_post_sets_urlencoded_content_type() {
        use menemen::form_data::FormData;

        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}/submit", port);

        let mut form = FormData::new();
        form.add("a", "b");

        let mut request = Request::new(&url, RequestTypes::POST).expect("build request");
        request.append_body(form.into());
        let _ = request.send();

        let raw = handle.join().expect("capture thread");
        let (_, headers) = head_of(&raw);
        assert_eq!(
            header_value(&headers, "Content-Type").as_deref(),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(header_value(&headers, "Content-Length").as_deref(), Some("3"));
    }

    #[test]
    fn every_method_reaches_the_request_line() {
        for (method, expected) in [
            (RequestTypes::GET, "GET"),
            (RequestTypes::POST, "POST"),
            (RequestTypes::PUT, "PUT"),
            (RequestTypes::DELETE, "DELETE"),
            (RequestTypes::HEAD, "HEAD"),
            (RequestTypes::PATCH, "PATCH"),
            (RequestTypes::OPTIONS, "OPTIONS"),
        ] {
            let (port, handle) = serve_and_capture(OK_RESPONSE);
            let url = format!("http://127.0.0.1:{}/x", port);
            let mut request = Request::new(&url, method).expect("build request");
            let _ = request.send();

            let raw = handle.join().expect("capture thread");
            let (request_line, _) = head_of(&raw);
            assert!(
                request_line.starts_with(&format!("{} /x ", expected)),
                "expected {expected}, got: {request_line}"
            );
        }
    }

    /// Redirects are followed by default, and a relative `Location` keeps the
    /// original port.
    #[test]
    fn redirects_are_followed_by_default() {
        let port = serve_sequence(vec![REDIRECT_RESPONSE, FINAL_RESPONSE]);
        let url = format!("http://127.0.0.1:{}/start", port);

        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let mut response = request.send().expect("send");

        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().expect("decode"), "final");
    }

    /// `set_follow_redirects(false)` must hand back the redirect itself.
    #[test]
    fn follow_redirects_can_be_disabled() {
        let port = serve_sequence(vec![REDIRECT_RESPONSE, FINAL_RESPONSE]);
        let url = format!("http://127.0.0.1:{}/start", port);

        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        request.set_follow_redirects(false);
        let response = request.send().expect("send");

        assert_eq!(response.response_info.status_code, 302);
        assert_eq!(
            header_value(
                &response
                    .headers
                    .iter()
                    .map(|h| format!("{}: {}", h.name, h.value))
                    .collect::<Vec<String>>(),
                "Location"
            )
            .as_deref(),
            Some("/final")
        );
    }

    /// A HEAD response has no body even though it advertises Content-Length,
    /// so reading it must return EOF rather than blocking.
    #[test]
    fn head_response_body_is_empty() {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}/x", port);
        let mut request = Request::new(&url, RequestTypes::HEAD).expect("build request");
        let mut response = request.send().expect("send HEAD");

        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().expect("decode"), "");
        let _ = handle.join();
    }
}

#[cfg(feature = "async")]
mod async_wire {
    use super::*;
    use menemen::request::{Request, RequestTypes};

    async fn get_raw(path: &str) -> String {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}{}", port, path);
        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let _ = request.send().await;
        handle.join().expect("capture thread")
    }

    #[tokio::test]
    async fn bodyless_get_sends_no_content_type() {
        let raw = get_raw("/api").await;
        let (_, headers) = head_of(&raw);
        assert_eq!(header_value(&headers, "Content-Type"), None, "got: {raw}");
    }

    #[tokio::test]
    async fn get_advertises_accept() {
        let raw = get_raw("/api").await;
        let (_, headers) = head_of(&raw);
        assert_eq!(header_value(&headers, "Accept").as_deref(), Some("*/*"));
    }

    #[tokio::test]
    async fn url_fragment_is_not_sent() {
        let raw = get_raw("/page#section").await;
        let (request_line, _) = head_of(&raw);
        assert!(!request_line.contains('#'), "{request_line}");
    }

    #[tokio::test]
    async fn query_without_a_path_is_sent() {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}?q=1&r=2", port);
        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let _ = request.send().await;
        let raw = handle.join().expect("capture thread");

        let (request_line, _) = head_of(&raw);
        assert!(request_line.contains("q=1&r=2"), "{request_line}");
    }

    #[tokio::test]
    async fn form_values_are_percent_encoded() {
        use menemen::form_data::FormData;

        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}/submit", port);

        let mut form = FormData::new();
        form.add("note", "a&b=c");
        form.add("msg", "hello world");

        let mut request = Request::new(&url, RequestTypes::POST).expect("build request");
        request.append_body(form.into());
        let _ = request.send().await;

        let raw = handle.join().expect("capture thread");
        assert_eq!(body_of(&raw), "note=a%26b%3Dc&msg=hello+world");
    }

    #[tokio::test]
    async fn redirects_are_followed_by_default() {
        let port = serve_sequence(vec![REDIRECT_RESPONSE, FINAL_RESPONSE]);
        let url = format!("http://127.0.0.1:{}/start", port);

        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        let mut response = request.send().await.expect("send");

        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().await.expect("decode"), "final");
    }

    #[tokio::test]
    async fn follow_redirects_can_be_disabled() {
        let port = serve_sequence(vec![REDIRECT_RESPONSE, FINAL_RESPONSE]);
        let url = format!("http://127.0.0.1:{}/start", port);

        let mut request = Request::new(&url, RequestTypes::GET).expect("build request");
        request.set_follow_redirects(false);
        let response = request.send().await.expect("send");

        assert_eq!(response.response_info.status_code, 302);
    }

    #[tokio::test]
    async fn head_response_body_is_empty() {
        let (port, handle) = serve_and_capture(OK_RESPONSE);
        let url = format!("http://127.0.0.1:{}/x", port);
        let mut request = Request::new(&url, RequestTypes::HEAD).expect("build request");
        let mut response = request.send().await.expect("send HEAD");

        assert_eq!(response.response_info.status_code, 200);
        assert_eq!(response.text().await.expect("decode"), "");
        let _ = handle.join();
    }
}
