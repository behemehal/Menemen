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
}
