
#[cfg(any(feature = "multipart", feature = "async"))]
mod multipart_form_data {
    use menemen::{body::Body, prelude::MultipartFormData};

    #[tokio::test]
    async fn test_multipart_form_data() {
        let mut form_data_builder = MultipartFormData::new();

        form_data_builder
            .add_file("key1", "./testData/file.txt")
            .await
            .expect("File not found");

        form_data_builder.add_string("key2", "value2".into());

        let body: Body = form_data_builder.into();
        let data = match body.body {
            menemen::body::BodyType::MultipartFormData(mut multipart) => {
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
}
