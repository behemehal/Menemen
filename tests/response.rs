#[cfg(test)]
mod url_test {
    use menemen::response::ResponseInfo;

    #[test]
    fn url_test() {
        let header = ResponseInfo::parse_response_info("HTTP/1.1 200 OK").unwrap();
        assert_eq!(header.status_code, 200);
        assert_eq!(header.status_message, "OK");
        assert_eq!(header.http_version, "HTTP/1.1");
    }

    #[test]
    fn parse_multi_word_status_message() {
        let header = ResponseInfo::parse_response_info("HTTP/1.1 301 Moved Permanently").unwrap();
        assert_eq!(header.status_code, 301);
        assert_eq!(header.status_message, "Moved Permanently");
        assert_eq!(header.http_version, "HTTP/1.1");
    }

    #[test]
    fn parse_without_reason_phrase() {
        let header = ResponseInfo::parse_response_info("HTTP/1.1 204").unwrap();
        assert_eq!(header.status_code, 204);
        assert_eq!(header.status_message, "");
        assert_eq!(header.http_version, "HTTP/1.1");
    }

    #[test]
    fn parse_fails_for_malformed_line() {
        let result = ResponseInfo::parse_response_info("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn parse_fails_for_non_numeric_status_code() {
        let result = ResponseInfo::parse_response_info("HTTP/1.1 abc OK");
        assert!(result.is_err());
    }
}
