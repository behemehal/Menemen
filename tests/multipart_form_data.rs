
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
