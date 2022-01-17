use menemen::request::{Request, RequestTypes};
use std::io::Read;

fn main() {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
    let mut response = request.send().unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer).unwrap();
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
}
