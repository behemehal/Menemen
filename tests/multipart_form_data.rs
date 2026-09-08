
#[cfg(feature = "multipart")]
mod multipart_form_data {
    use menemen::{body::Body, prelude::MultipartFormData};

    #[tokio::test]
    async fn test_multipart_form_data() {
        let mut form_data_builder = MultipartFormData::new();

        #[cfg(feature = "async")]
        form_data_builder
            .add_file("key1", "./testData/file.txt")
            .await
            .expect("File not found");

        #[cfg(not(feature = "async"))]
        form_data_builder
            .add_file("key1", "./testData/file.txt")
            .expect("File not found");

        form_data_builder.add_string("key2", "value2".into());

        let body: Body = form_data_builder.into();
        let data = match body.body {
            menemen::body::BodyType::MultipartFormData(mut multipart) => {
                #[cfg(feature = "async")]
                let built = multipart
                    .build()
                    .await
                    .expect("Error building multipart form");

                #[cfg(not(feature = "async"))]
                let built = multipart.build().expect("Error building multipart form");

                let buf_str = String::from_utf8(built).unwrap();
                buf_str
            }
            _ => panic!("Body is not MultipartFormData"),
        };

        let boundary = data[2..36].to_string();
        let correct_data = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"key1\"; filename=\"file.txt\"\r\nContent-Type: text/plain\r\n\r\nHello From file.txt\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"key2\"\r\n\r\nvalue2\r\n--{boundary}--\r\n");
        assert_eq!(data, correct_data);
    }

    /// Builds a form to its wire representation.
    async fn build_to_string(form_data_builder: MultipartFormData) -> String {
        let body: Body = form_data_builder.into();
        match body.body {
            menemen::body::BodyType::MultipartFormData(mut multipart) => {
                #[cfg(feature = "async")]
                let built = multipart.build().await.expect("build multipart");

                #[cfg(not(feature = "async"))]
                let built = multipart.build().expect("build multipart");

                String::from_utf8(built).expect("utf8")
            }
            _ => panic!("Body is not MultipartFormData"),
        }
    }

    /// Regression: the filename was taken with `split('/')`, so on Windows —
    /// where paths use backslashes — nothing split and the caller's entire
    /// absolute path went out in `Content-Disposition`, leaking their
    /// username and directory layout to the server.
    #[tokio::test]
    async fn file_part_sends_only_the_base_name() {
        let separator = std::path::MAIN_SEPARATOR;
        let path = format!(".{separator}testData{separator}file.txt");

        let mut form_data_builder = MultipartFormData::new();

        #[cfg(feature = "async")]
        form_data_builder
            .add_file("doc", &path)
            .await
            .expect("add file");

        #[cfg(not(feature = "async"))]
        form_data_builder.add_file("doc", &path).expect("add file");

        let text = build_to_string(form_data_builder).await;

        assert!(
            text.contains("filename=\"file.txt\""),
            "expected a bare filename, got: {text}"
        );
        assert!(
            !text.contains("testData"),
            "the local path leaked into the request: {text}"
        );
    }

    /// A quote or CRLF in a field name must not be able to close the quoted
    /// string or start a new header line.
    #[tokio::test]
    async fn field_names_cannot_inject_headers() {
        let mut form_data_builder = MultipartFormData::new();
        form_data_builder.add_string("a\"b\r\nX-Injected: yes", "value".to_string());

        let text = build_to_string(form_data_builder).await;

        assert!(
            !text.contains("\r\nX-Injected"),
            "field name injected a header line: {text}"
        );
        assert!(
            text.contains("name=\"a\\\"bX-Injected: yes\""),
            "quote was not escaped: {text}"
        );
    }

    /// Derives the boundary from a built body, since the field is crate-private.
    async fn built_boundary() -> String {
        let mut form_data_builder = MultipartFormData::new();
        form_data_builder.add_string("key", "value".into());

        let body: Body = form_data_builder.into();
        let built = match body.body {
            menemen::body::BodyType::MultipartFormData(mut multipart) => {
                #[cfg(feature = "async")]
                let built = multipart.build().await.expect("build multipart");

                #[cfg(not(feature = "async"))]
                let built = multipart.build().expect("build multipart");

                built
            }
            _ => panic!("Body is not MultipartFormData"),
        };

        let text = String::from_utf8(built).expect("utf8");
        // The body opens with "--" followed by the boundary and a CRLF.
        text[2..text.find("\r\n").expect("CRLF after boundary")].to_string()
    }

    /// RFC 2046 restricts which characters may appear in a boundary. The
    /// generator used to sample raw code points 48..122, which let through
    /// ;<>@[\]^` and never produced 'z'.
    #[tokio::test]
    async fn generated_boundary_only_uses_safe_characters() {
        for _ in 0..50 {
            let boundary = built_boundary().await;

            assert!(boundary.starts_with("----"), "unexpected prefix: {boundary}");
            assert_eq!(boundary.len(), 34, "unexpected length: {boundary}");

            for character in boundary.chars() {
                assert!(
                    character.is_ascii_alphanumeric() || character == '-',
                    "boundary {boundary} contains invalid character {character:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn generated_boundaries_differ_between_instances() {
        assert_ne!(built_boundary().await, built_boundary().await);
    }
}
