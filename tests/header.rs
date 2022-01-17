#[cfg(test)]
mod header_test {
    use menemen::request::Header;

    #[test]
    fn parse_header() {
        let header = Header::parse("Content-Type: text/html; charset=utf-8").unwrap();
        assert_eq!(header.name.clone(), "Content-Type");
        assert_eq!(header.value, "text/html; charset=utf-8");
    }
}
