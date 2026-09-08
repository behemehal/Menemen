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

    /// Regression: values were interpolated raw, so a value containing `&` or
    /// `=` silently became extra fields on the server.
    #[test]
    fn reserved_characters_are_escaped() {
        let mut form = FormData::new();
        form.add("note", "a&b=c");
        form.add("msg", "hello world");

        let mut buffer = Vec::new();
        form.read_to_end(&mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "note=a%26b%3Dc&msg=hello+world"
        );
    }

    #[test]
    fn non_ascii_values_are_utf8_percent_encoded() {
        let mut form = FormData::new();
        form.add("city", "İzmir");

        let mut buffer = Vec::new();
        form.read_to_end(&mut buffer).unwrap();
        // İ is U+0130, which is 0xC4 0xB0 in UTF-8.
        assert_eq!(String::from_utf8(buffer).unwrap(), "city=%C4%B0zmir");
    }

    /// Encoding and parsing must be inverses, including for values that contain
    /// the separators themselves.
    #[test]
    fn encode_then_parse_round_trips() {
        let mut form = FormData::new();
        form.add("note", "a&b=c");
        form.add("msg", "hello world");
        form.add("city", "İzmir");
        form.add("empty", "");

        let mut buffer = Vec::new();
        form.read_to_end(&mut buffer).unwrap();
        let encoded = String::from_utf8(buffer).unwrap();

        let parsed = FormData::from_str(&encoded);
        assert_eq!(parsed.form_data.len(), 4);
        assert_eq!(parsed.get("note"), Some(&"a&b=c".to_string()));
        assert_eq!(parsed.get("msg"), Some(&"hello world".to_string()));
        assert_eq!(parsed.get("city"), Some(&"İzmir".to_string()));
        assert_eq!(parsed.get("empty"), Some(&"".to_string()));
    }

    /// Regression: the `Read` impl was stateless, so every call rebuilt the
    /// payload and returned it from the start. `read_to_end` never saw EOF and
    /// grew until it exhausted memory.
    #[test]
    fn read_reaches_eof() {
        let mut form = FormData::new();
        form.add("a", "1");

        let mut first = [0u8; 3];
        assert_eq!(form.read(&mut first).unwrap(), 3);
        assert_eq!(&first, b"a=1");

        let mut second = [0u8; 3];
        assert_eq!(
            form.read(&mut second).unwrap(),
            0,
            "a second read must report EOF, not restart the payload"
        );
    }

    /// Reading in small pieces must reassemble the payload exactly once.
    #[test]
    fn partial_reads_do_not_duplicate_content() {
        let mut form = FormData::new();
        form.add("key", "value");
        form.add("other", "thing");

        let mut collected = Vec::new();
        let mut chunk = [0u8; 4];
        loop {
            match form.read(&mut chunk).unwrap() {
                0 => break,
                n => collected.extend_from_slice(&chunk[..n]),
            }
        }

        assert_eq!(
            String::from_utf8(collected).unwrap(),
            "key=value&other=thing"
        );
    }

    /// The payload must be encoded once and streamed, not rebuilt per call.
    ///
    /// Rebuilding per call is quadratic: this form encodes to ~40 KB, so
    /// reading it a byte at a time would re-encode 40 KB roughly 40,000 times
    /// (~1.6 GB of allocation traffic). Measured, that takes ~12.7s against
    /// under 10ms for the correct implementation, so the budget below leaves
    /// plenty of slack for a slow machine while still catching a regression.
    #[test]
    fn payload_is_encoded_once_not_per_read() {
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut form = FormData::new();
            for index in 0..200 {
                form.add(&format!("field{index}"), &"x".repeat(190));
            }

            let mut total = 0usize;
            let mut byte = [0u8; 1];
            while form.read(&mut byte).unwrap() == 1 {
                total += 1;
            }

            let _ = sender.send(total);
        });

        match receiver.recv_timeout(Duration::from_secs(4)) {
            Ok(total) => assert!(total > 39_000, "unexpected payload size: {total}"),
            Err(_) => panic!("timed out: the payload is being re-encoded on every read"),
        }
    }

    /// Mutating the form after reading has started restarts the stream from the
    /// new content rather than splicing old and new bytes together.
    #[test]
    fn mutation_restarts_the_stream() {
        let mut form = FormData::new();
        form.add("a", "1");

        let mut first = [0u8; 1];
        assert_eq!(form.read(&mut first).unwrap(), 1);
        assert_eq!(&first, b"a");

        form.add("b", "2");

        let mut rest = Vec::new();
        form.read_to_end(&mut rest).unwrap();
        assert_eq!(String::from_utf8(rest).unwrap(), "a=1&b=2");
    }

    /// A malformed escape must be preserved rather than dropped.
    #[test]
    fn invalid_escape_is_kept_verbatim() {
        let form = FormData::from_str("a=%zz&b=%4");
        assert_eq!(form.get("a"), Some(&"%zz".to_string()));
        assert_eq!(form.get("b"), Some(&"%4".to_string()));
    }
}
