use menemen::request::{ContentTypes, Request, RequestTypes};
use std::{fs::File, io::Read};

fn main() {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST).unwrap();

    //Read file
    let mut file = File::open("./examples/post.json").unwrap();
    request.content_type = ContentTypes::JSON;
    let mut response = request.send_with_body(&mut file).unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer).unwrap();
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
    println!("Response info: {:?}", response.response_info);
}
