#[cfg(test)]
mod form_data_test {
    use menemen::form_data::FormData;
    use std::io::Read;

    #[test]
    fn from_str_handles_missing_values_without_panicking() {
        let form = FormData::from_str("a&b=1&c=");
        assert_eq!(form.form_data.len(), 3);
        assert_eq!(form.form_data[0], ("a".to_string(), "".to_string()));
        assert_eq!(form.form_data[1], ("b".to_string(), "1".to_string()));
        assert_eq!(form.form_data[2], ("c".to_string(), "".to_string()));
    }

    #[test]
    fn build_returns_empty_for_empty_form_data() {
        let mut form = FormData::new();
        let mut buf = [0u8; 16];
        let read = form.read(&mut buf).unwrap();
        assert_eq!(read, 0);
    }
}
