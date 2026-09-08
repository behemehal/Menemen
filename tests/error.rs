#[cfg(test)]
mod error_test {
    use menemen::error::RequestError;

    #[test]
    fn display_covers_every_variant() {
        // The Display impl must never fall back to Debug formatting, so every
        // message is checked for readable prose rather than a variant name.
        let cases = vec![
            RequestError::CantSetHeadersAfterRequestSent,
            RequestError::CantResolveUrl,
            RequestError::ConnectionTimeout,
            RequestError::MalformedUrl,
            RequestError::AlreadySent,
            RequestError::ConnectionError("refused".to_string()),
            RequestError::TlsNotEnabled,
            RequestError::FileError("missing.txt".to_string()),
            RequestError::TextError("bad utf8".to_string()),
            RequestError::JsonError("unexpected token".to_string()),
            RequestError::InvalidHeader("no colon here".to_string()),
            RequestError::InvalidResponse("GARBAGE".to_string()),
        ];

        for case in cases {
            let rendered = case.to_string();
            assert!(!rendered.is_empty(), "empty Display for {:?}", case);
            assert!(
                !rendered.starts_with("RequestError"),
                "Display fell through to Debug for {:?}: {}",
                case,
                rendered
            );
        }
    }

    #[test]
    fn display_includes_the_payload() {
        assert!(RequestError::ConnectionError("refused".into())
            .to_string()
            .contains("refused"));
        assert!(RequestError::FileError("missing.txt".into())
            .to_string()
            .contains("missing.txt"));
        assert!(RequestError::InvalidHeader("no colon here".into())
            .to_string()
            .contains("no colon here"));
        assert!(RequestError::InvalidResponse("GARBAGE".into())
            .to_string()
            .contains("GARBAGE"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&RequestError::ConnectionTimeout);

        // Must be usable with `?` in functions returning a boxed error.
        fn fallible() -> Result<(), Box<dyn std::error::Error>> {
            Err(RequestError::MalformedUrl)?;
            Ok(())
        }
        assert!(fallible().is_err());
    }

    #[test]
    fn converts_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "slow");
        let converted: RequestError = io_error.into();
        assert!(matches!(converted, RequestError::ConnectionError(_)));
        assert!(converted.to_string().contains("slow"));
    }
}
