//! Demonstrates multipart form-data body creation in async and blocking modes.

#[cfg(feature = "async")]
use menemen::multipart_form::MultipartFormData;

#[cfg(not(feature = "async"))]
fn main() {
    use std::io;

    use menemen::{body::Body, form_data::FormData};

    let mut form_data_builder = FormData::new();

    form_data_builder.set("key", "value");

    let mut body: Body = form_data_builder.into();

    println!(
        "Body built, waiting for 25 seconds copy everything to array: {:?}",
        body.size_hint()
    );

    let buffer: Vec<u8> = Vec::new();

    println!("Buffer: {:?}", buffer);

    let buff: Vec<u8> = Vec::new();
    let mut cursor = io::Cursor::new(buff);

    io::copy(&mut body, &mut cursor).unwrap();

    println!("read_to_end: {:?}", cursor.get_ref());
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use menemen::body::Body;

    let mut form_data_builder = MultipartFormData::new();

    form_data_builder
        .add_file("key1", "./testData/file.txt")
        .await?;
    form_data_builder.add_string("key2", "value2".into());

    let body: Body = form_data_builder.into();

    match body.body {
        menemen::body::BodyType::Reader(_) => unreachable!(),
        menemen::body::BodyType::Bytes(_) => unreachable!(),
        menemen::body::BodyType::FormData(_) => unreachable!(),
        menemen::body::BodyType::MultipartFormData(mut multipart) => {
            let built = multipart.build().await?;
            let buf_str = String::from_utf8(built).unwrap();
            println!("Built: \n{:?}", buf_str);
        }
    }

    Ok(())
}
