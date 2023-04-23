use libflate::gzip::Decoder;
use menemen::request::{Request, RequestTypes};
use std::io::Read;

fn main() {
    let mut request = Request::new("http://behemehal.org", RequestTypes::GET).unwrap();
    request.set_header(&"Accept-Encoding", &"gzip");

    let mut response = request.send().unwrap();

    println!("Response info: {:?}", response.response_info);
    println!("Response headers: {:?}", response.headers);

    // Pipe response stream through gzip decoder
    let mut decoder = Decoder::new(&mut response.stream).unwrap();

    let mut text_buffer = Vec::new();

    // Read decoded response into text buffer
    decoder.read_to_end(&mut text_buffer).unwrap();

    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
}
