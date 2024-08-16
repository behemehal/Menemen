use menemen::request::{ContentTypes, Request, RequestTypes};

#[cfg(not(feature = "async"))]
use std::{fs::File, io::Read};

#[cfg(feature = "async")]
use tokio::fs::File;

#[cfg(not(feature = "async"))]
fn main() {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST).unwrap();

    let mut file = File::open("./examples/post.json").unwrap();
    request.body(
        file.into()
    );

    //Read file
    request.content_type = ContentTypes::JSON;
    let mut response = request.send_with_body(&mut file).unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer).unwrap();
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
    println!("Response info: {:?}", response.response_info);
}

#[cfg(feature = "async")]
async fn main() {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST).unwrap();

    let mut file = File::open("./examples/post.json").await.unwrap();
    request.body(
        
    )

    //Read file
    request.content_type = ContentTypes::JSON;
    let mut response = request.send_with_body(&mut file).await.unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer).await.unwrap();
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
    println!("Response info: {:?}", response.response_info);
}