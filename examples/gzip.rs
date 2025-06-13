use menemen::request::{Request, RequestTypes};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://behemehal.org", RequestTypes::GET).unwrap();
    request.set_header(&"Accept-Encoding", &"gzip");

    let mut response = request.send()?;

    println!("Response info: {:?}", response.response_info);
    println!("Response headers: {:?}", response.headers);

    let text = response.text()?;
    println!("Text: {}", text);

    Ok(())
}
