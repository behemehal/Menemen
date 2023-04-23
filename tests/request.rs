#[cfg(test)]
mod request_test {
    use menemen::request::{Request, RequestTypes};

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
}
