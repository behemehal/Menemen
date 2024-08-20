#[cfg(feature = "async")]
use tokio::{fs::File, io::AsyncRead, time::sleep};

use std::time::Duration;

#[cfg(not(feature = "async"))]
use std::{fs::File, io::Read};

use std::any::Any;

use menemen::multipart_form::MultipartFormData;

#[cfg(not(feature = "async"))]
fn main() {
    use std::{io, thread::sleep};

    use menemen::{body::Body, form_data::FormData};

    let mut form_data_builder = FormData::new();

    form_data_builder.set("key", "value");

    let mut body: Body = form_data_builder.into();

    println!(
        "Body built, waiting for 25 seconds copy everything to array: {:?}",
        body.size_hint()
    );

    let mut buffer: Vec<u8> = Vec::new();

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

    //form_data_builder.add_file("key1", "./20MB.zip").await?;

    form_data_builder.add_string("key2", "value2".into());

    println!(
        "First file oppened, Check the io and memory usage of the program, sleep for 5 seconds"
    );


    let mut body: Body = form_data_builder.into();

    println!("Body built, waiting for 25 seconds copy everything to array");

    //let mut buffer = Vec::new();

    match body.body {
        menemen::body::BodyType::Reader(_) => todo!(),
        menemen::body::BodyType::Bytes(_) => todo!(),
        menemen::body::BodyType::FormData(_) => todo!(),
        menemen::body::BodyType::MultipartFormData(mut multipart) => {
            let built = multipart.build().await?;

            let buf_str = String::from_utf8(built).unwrap();

            println!("Built: \n{}", buf_str);
        }
    }

    loop {
        sleep(Duration::from_secs(1));
    }
}
