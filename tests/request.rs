#[cfg(test)]
mod request_test {
    use menemen::request::{Request, RequestTypes};

    #[test]
    fn body_build_test() {
        let mut request = Request::new("https://behemehal.net/test", RequestTypes::GET).unwrap();
        let body = request.build_request_body();
        assert_eq!(body, "GET https://behemehal.net:443/test HTTP/1.1\r\nHost:behemehal.net:443\r\nConnection:close\r\nCache-Control:max-age=0\r\nUser-Agent:Menemen/0.2.0-alpha\r\nContent-Type:text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\r\n".to_string());
    }

    #[test]
    fn get_set_header_test() {
        let mut request = Request::new("https://behemehal.net/test", RequestTypes::GET).unwrap();
        assert!(request.set_header("key", "value").is_none());
        assert!(matches!(
            request.get_header("key"), Some(e) if e.name == "key" && e.value == "value"
        ));
    }

    #[test]
    fn timeout_set() {
        let mut request = Request::new("https://behemehal.net/test", RequestTypes::GET).unwrap();
        assert!(request.set_timeout(100).is_none());
    }
}
