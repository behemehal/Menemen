#[cfg(test)]
mod header_test {
    use menemen::request::Header;

    #[test]
    fn parse_header() {
        let header = Header::parse("Content-Type: text/html; charset=utf-8").unwrap();
        assert_eq!(header.name.clone(), "Content-Type");
        assert_eq!(header.value, "text/html; charset=utf-8");
    }

    #[test]
    fn parse_header_with_empty_value() {
        let header = Header::parse("X-Trace-Id: ").unwrap();
        assert_eq!(header.name, "X-Trace-Id");
        assert_eq!(header.value, "");
    }

    #[test]
    fn parse_header_with_colon_in_value() {
        let header = Header::parse("Time: 10:20:30").unwrap();
        assert_eq!(header.name, "Time");
        assert_eq!(header.value, "10:20:30");
    }

    #[test]
    fn parse_header_without_space_after_colon() {
        let header = Header::parse("X-Test:value").unwrap();
        assert_eq!(header.name, "X-Test");
        assert_eq!(header.value, "value");
    }

    #[test]
    fn parse_header_fails_without_colon() {
        let header = Header::parse("MalformedHeader");
        assert!(header.is_err());
    }
}
