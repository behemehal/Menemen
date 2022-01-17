#[cfg(test)]
mod url_test {
    use menemen::url::Url;

    #[test]
    fn url_test() {
        let url = Url::build_from_string("https://behemehal.net/test?qtest=123".to_string()).unwrap();
        
        assert_eq!(url.is_https, true);
        assert_eq!(url.host, "behemehal.net".to_string());
        assert_eq!(url.query_params.len(), 1);
        assert_eq!(url.query_params[0].name, "qtest".to_string());
        assert_eq!(url.query_params[0].value, "123".to_string());
        assert_eq!(url.port, 443);
        assert_eq!(url.paths.len(), 1);
        assert_eq!(url.paths[0], "test".to_string());
    }
}