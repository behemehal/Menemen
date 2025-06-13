use menemen::request::{ContentTypes, Request, RequestTypes};

#[cfg(not(feature = "async"))]
use std::{fs::File, io::Read};

#[cfg(feature = "async")]
use tokio::fs::File;

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use menemen::body::Body;

    //Create a new request
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST).unwrap();

    //Read file, and append it to the request
    let file = File::open("./examples/post.json").unwrap();
    request.append_body(file.into());

    //Set content type
    request.content_type = ContentTypes::JSON; 

    let mut response = request.send()?;

    let text = response.text()?;
    println!("Text: {}", text);
    println!("Response info: {:?}", response.response_info);
    Ok(())
}

#[cfg(feature = "async")]
async fn main() {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST).unwrap();

    let mut file = File::open("./examples/post.json").await.unwrap();
    //Read file
    request.content_type = ContentTypes::JSON;
    let mut response = request.send_with_body(&mut file).await.unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer).await.unwrap();
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
    println!("Response info: {:?}", response.response_info);
}