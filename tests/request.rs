#[cfg(test)]
mod request_test {
    use menemen::request::{ContentTypes, Request, RequestTypes};

    #[test]
    fn get_set_header_test() {
        let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
        assert!(request.set_header("key", "value").is_none());
        assert!(matches!(
            request.get_header("key"), Some(e) if e.name == "key" && e.value == "value"
        ));
    }

    #[test]
    fn timeout_set() {
        let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
        assert!(request.set_timeout(100).is_none());
    }

    #[test]
    fn request_new_sets_default_headers() {
        let request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();

        assert!(matches!(
            request.get_header("Host"),
            Some(h) if h.value == "behemehal.org"
        ));
        assert!(matches!(
            request.get_header("Connection"),
            Some(h) if h.value == "close"
        ));
        assert!(matches!(
            request.get_header("Cache-Control"),
            Some(h) if h.value == "max-age=0"
        ));
        assert!(matches!(
            request.get_header("User-Agent"),
            Some(h) if h.value.starts_with("Menemen/")
        ));
    }

    #[test]
    fn request_new_sets_host_with_custom_port() {
        let request = Request::new("http://example.com:8080/test", RequestTypes::GET).unwrap();
        assert!(matches!(
            request.get_header("Host"),
            Some(h) if h.value == "example.com:8080"
        ));
    }

    #[test]
    fn set_header_overwrites_existing_value() {
        let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
        assert!(request.set_header("Accept", "text/plain").is_none());
        assert!(request.set_header("Accept", "application/json").is_none());

        let headers = request.get_headers();
        let accept_count = headers.iter().filter(|h| h.name == "Accept").count();
        assert_eq!(accept_count, 1);
        assert!(matches!(
            request.get_header("Accept"),
            Some(h) if h.value == "application/json"
        ));
    }

    #[test]
    fn content_types_return_expected_mime_types() {
        assert_eq!(ContentTypes::JSON.get_type(), "application/json");
        assert_eq!(ContentTypes::MultipartFormData.get_type(), "multipart/form-data");
        assert_eq!(ContentTypes::FormData.get_type(), "application/x-www-form-urlencoded");
    }

    #[test]
    fn set_header_rejects_crlf_injection() {
        let mut request = Request::new("https://behemehal.org/test", RequestTypes::GET).unwrap();
        assert!(request.set_header("X-Test", "safe\r\nInjected: bad").is_some());
        assert!(request.get_header("X-Test").is_none());
    }
}
