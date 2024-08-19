use menemen::multipart_form_data::MultipartFormData;

#[cfg(feature = "async")]
use tokio::{fs::File, io::AsyncRead};

#[cfg(not(feature = "async"))]
use std::{fs::File, io::Read};

use std::any::Any;

#[cfg(not(feature = "async"))]
fn main() {
    use std::{io, thread::sleep};

    use menemen::{body::Body, form_data::FormData};

    let mut form_data_builder = FormData::new();

    form_data_builder.set("key", "value");

    let mut body: Body = form_data_builder.into();

    println!("Body built, waiting for 25 seconds copy everything to array: {:?}", body.size_hint());

    let mut buffer : Vec<u8> = Vec::new();

    println!("Buffer: {:?}", buffer);

    
    let buff: Vec<u8> = Vec::new();
    let mut cursor = io::Cursor::new(buff);

    io::copy(&mut body, &mut cursor).unwrap();


    println!("read_to_end: {:?}", cursor.get_ref());

}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use std::thread::sleep;

    use menemen::{body::Body, form_data::FormData};
    use tokio::io::AsyncReadExt;

    let mut form_data_builder = MultipartFormData::new();

    println!("Sleeping for 15 seconds take your time");

    sleep(std::time::Duration::from_secs(15));

    let file = File::open("./20MB.zip").await.unwrap();
    form_data_builder.add("key1", file);

    println!(
        "First file oppened, Check the io and memory usage of the program, sleep for 15 seconds"
    );

    sleep(std::time::Duration::from_secs(15));

    let second_file = File::open("./1GB.zip").await.unwrap();
    form_data_builder.add("key2", second_file);

    println!("Second File added waiting 15secs until body is built");

    sleep(std::time::Duration::from_secs(15));

    let mut body: Body = form_data_builder.into();

    println!("Body built, waiting for 25 seconds copy everything to array");

    sleep(std::time::Duration::from_secs(25));

    let mut buffer = Vec::new();

    body.read_to_end(&mut buffer).await.unwrap();

    loop {
        sleep(std::time::Duration::from_secs(1));
    }
}
